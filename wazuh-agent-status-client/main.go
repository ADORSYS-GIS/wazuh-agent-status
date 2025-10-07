package main

import (
	"bufio"
	"embed"
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"time"

	"github.com/getlantern/systray"
	"gopkg.in/natefinch/lumberjack.v2"
)

//go:embed assets/*
var embeddedFiles embed.FS

var (
	statusItem, connectionItem, updateItem, versionItem *systray.MenuItem
	enabledIcon, disabledIcon                           []byte

	// monitorOnce ensures we only start the monitoring routines once
	monitorOnce sync.Once

	// updateMutex prevents concurrent attempts to start the update stream
	updateMutex sync.Mutex
)

// Version is set at build time via ldflags
var Version = "v0.3.7-auto-update"

// AUTH_TOKEN must match the server's token
const AUTH_TOKEN = "SUPER_SECRET_LOCAL_TOKEN"

func getUserLogFilePath() string {
	var logDir string

	switch runtime.GOOS {
	case "linux", "darwin":
		logDir = filepath.Join(os.Getenv("HOME"), ".wazuh")
	case "windows":
		logDir = filepath.Join(os.Getenv("APPDATA"), "wazuh", "logs")
	default:
		logDir = "./logs"
	}

	// Ensure the directory exists
	if err := os.MkdirAll(logDir, 0755); err != nil {
		log.Fatalf("failed to create log directory: %v", err)
	}

	return filepath.Join(logDir, "wazuh-agent-status-client.log")
}

func init() {
	logFilePath := getUserLogFilePath()

	log.SetOutput(&lumberjack.Logger{
		Filename:   logFilePath,
		MaxSize:    10,
		MaxBackups: 3,
		MaxAge:     30,
		Compress:   true,
	})
}

// Main entry point
func main() {
	for _, arg := range os.Args[1:] {
		if arg == "--version" || arg == "-v" {
			fmt.Println(Version)
			return
		}
	}
	log.Printf("Starting frontend... (version: %s)", Version)
	systray.Run(onReady, onExit)
}

// onReady sets up the tray icon
func onReady() {
	// Load icons and set up systray
	mainIcon, err := getEmbeddedFile(getIconPath())
	if err != nil {
		log.Fatalf("Failed to load main icon: %v", err)
	}
	systray.SetIcon(mainIcon)
	systray.SetTooltip("Wazuh Agent Status")

	enabledIcon, err = getEmbeddedFile("assets/green-dot.png")
	if err != nil {
		log.Printf("Failed to load enabled icon: %v", err)
	}
	disabledIcon, err = getEmbeddedFile("assets/gray-dot.png")
	if err != nil {
		log.Printf("Failed to load disabled icon: %v", err)
	}

	// Create menu items
	statusItem = systray.AddMenuItem("Agent: Unknown", "Wazuh Agent Status")
	statusItem.Disable()
	connectionItem = systray.AddMenuItem("Connection: Unknown", "Wazuh Agent Connection")
	connectionItem.Disable()
	systray.AddSeparator()
	updateItem = systray.AddMenuItem("---", "Update the Wazuh Agent")
	updateItem.Disable() // Initially disabled
	systray.AddSeparator()
	versionItem = systray.AddMenuItem("v---", "The version state of the wazuhbsetup")
	versionItem.Disable() // Initially disabled

	// Start background monitoring routines (guarded to run only once)
	monitorOnce.Do(func() {
		go monitorStatusStream()
		go monitorVersion() // Handles both startup and periodic checks
		go goroutineCounter()
	})

	// Handle menu item clicks
	go handleMenuActions()
}

// goroutineCounter logs goroutine count periodically to help detect leaks.
func goroutineCounter() {
	for {
		time.Sleep(1 * time.Minute)
		log.Printf("Debug: goroutines=%d", runtime.NumGoroutine())
	}
}

// monitorStatusStream establishes a persistent connection to receive status updates.
func monitorStatusStream() {
	for {
		conn, err := net.DialTimeout("tcp", "localhost:50505", 5*time.Second)
		if err != nil {
			log.Printf("Failed to connect to backend for status stream: %v. Retrying in 10s...", err)
			updateStatusItems("Unknown", "Unknown")
			time.Sleep(10 * time.Second)
			continue
		}

		log.Println("Status stream connected. Sending subscription...")
		fmt.Fprintln(conn, "auth "+AUTH_TOKEN)
		fmt.Fprintln(conn, "subscribe-status")

		reader := bufio.NewReader(conn)

		for {
			response, err := reader.ReadString('\n')
			if err != nil {
				log.Printf("Status stream closed by server or error: %v. Reconnecting...", err)
				conn.Close()
				break
			}

			response = strings.TrimSpace(response)
			if strings.HasPrefix(response, "STATUS_UPDATE:") {
				parts := strings.Split(response, ": ")
				if len(parts) > 1 {
					data := strings.Split(parts[1], ", ")
					if len(data) == 2 {
						updateStatusItems(data[0], data[1])
					}
				}
			} else if strings.HasPrefix(response, "ERROR:") {
				log.Printf("Error from status stream: %s", response)
				conn.Close()
				break
			}
		}
		time.Sleep(5 * time.Second)
	}
}

// updateStatusItems safely updates the systray menu items.
func updateStatusItems(status, connection string) {
	if status == "Active" {
		statusItem.SetTitle("Agent: Active")
		statusItem.SetIcon(enabledIcon)
	} else {
		statusItem.SetTitle("Agent: Inactive")
		statusItem.SetIcon(disabledIcon)
	}

	if connection == "Connected" {
		connectionItem.SetTitle("Connection: Connected")
		connectionItem.SetIcon(enabledIcon)
	} else {
		connectionItem.SetTitle("Connection: Disconnected")
		connectionItem.SetIcon(disabledIcon)
	}
}

// monitorVersion handles the startup check and the periodic check.
func monitorVersion() {
	// Initial check on startup (autoStart=true triggers immediate update if needed)
	handleVersionCheck(true) 
	
	// Periodic check every 4 hours
	for {
		time.Sleep(4 * time.Hour)
		handleVersionCheck(true) // autoStart=true ensures automatic update initiation
	}
}

// handleVersionCheck communicates with the backend, updates the menu, and conditionally starts an update.
// The 'autoStart' flag determines if an available update should be automatically triggered.
func handleVersionCheck(autoStart bool) {
	response, err := sendCommandAndReceive("get-version")
	if err != nil {
		log.Printf("Error fetching version: %v", err)
		versionItem.SetTitle("Version: Unknown")
		updateItem.SetTitle("---")
		updateItem.Disable()
		return
	}

	response = strings.TrimPrefix(response, "VERSION_CHECK: ")
	isOutdated := strings.Contains(response, "Outdated")
	
	// --- Update Menu Items ---
	if isOutdated {
		// Extract version number part only
		parts := strings.Split(response, ", ")
		version := "vUnknown"
		if len(parts) == 2 {
			version = parts[1]
		}
		versionItem.SetTitle(version)
		updateItem.SetTitle("Update Available")
		updateItem.Enable() // Always enable manual trigger
		
		if autoStart {
			log.Println("Automatic update triggered by periodic check.")
			updateItem.Disable()
			go startUpdateStream()
		}
		
	} else if strings.Contains(response, "Up to date") {
		// Extract version number part only
		parts := strings.Split(response, ", ")
		version := "vUnknown"
		if len(parts) == 2 {
			version = parts[1]
		}
		versionItem.SetTitle(version)
		updateItem.SetTitle("Up to date")
		updateItem.Disable()
	} else {
		versionItem.SetTitle(response)
		updateItem.SetTitle("---")
		updateItem.Disable()
	}
}

// handleMenuActions listens for menu item clicks and performs actions
func handleMenuActions() {
	for range updateItem.ClickedCh {
		log.Println("Update clicked. Starting stream...")
		updateItem.SetTitle("Starting Update...")
		updateItem.Disable()
		// Manual trigger bypasses the autoStart logic
		go startUpdateStream() 
	}
}

// startUpdateStream initiates the update on the server and streams the progress back.
func startUpdateStream() {
    // Use mutex to prevent multiple update streams from starting simultaneously
    if !updateMutex.TryLock() {
        log.Println("Update already in progress. Skipping.")
        return
    }
    defer updateMutex.Unlock() // This runs LAST

    conn, err := net.DialTimeout("tcp", "localhost:50505", 5*time.Second)
    if err != nil {
        log.Printf("Failed to connect to backend for update stream: %v", err)
        updateItem.SetTitle("Update Failed (Connect)")
        updateItem.Enable() 
        return
    }
    defer conn.Close()

    fmt.Fprintln(conn, "auth "+AUTH_TOKEN)
    fmt.Fprintln(conn, "update")

    reader := bufio.NewReader(conn)
    
    // Read and process the streaming response from the server
    for {
        response, err := reader.ReadString('\n')
        if err != nil {
            if err != io.EOF {
                log.Printf("Update stream error: %v", err)
                updateItem.SetTitle("Update Failed (Stream)")
            } else {
                log.Println("Update stream finished (EOF).")
                // Keep the temporary status set:
                updateItem.SetTitle("Checking Version Status...") 
            }
            break
        }

        trimmed := strings.TrimSpace(response)
        log.Printf("Update Stream: %s", trimmed)

        // Display status on the menu item
        if strings.HasPrefix(trimmed, "UPDATE_PROGRESS:") {
            status := strings.TrimPrefix(trimmed, "UPDATE_PROGRESS: ")
			if status == "Complete" {
				break 
			}
            updateItem.SetTitle(fmt.Sprintf("Updating: %s", status))
        }
    }
    
    handleVersionCheck(false)
}

// sendCommandAndReceive is a general utility for single request/response commands.
func sendCommandAndReceive(command string) (string, error) {
	conn, err := net.DialTimeout("tcp", "localhost:50505", 3*time.Second)
	if err != nil {
		return "", fmt.Errorf("failed to dial: %w", err)
	}
	defer conn.Close()

	_ = conn.SetReadDeadline(time.Now().Add(5 * time.Second))

	_, err = fmt.Fprintln(conn, "auth "+AUTH_TOKEN)
	if err != nil {
		return "", fmt.Errorf("failed to authorize: %w", err)
	}

	_, err = fmt.Fprintln(conn, command)
	if err != nil {
		return "", fmt.Errorf("failed to send command: %w", err)
	}

	reader := bufio.NewReader(conn)
	response, err := reader.ReadString('\n')
	if err != nil {
		return "", fmt.Errorf("failed to read response: %w", err)
	}

	trimmed := strings.TrimSpace(response)
	if strings.HasPrefix(trimmed, "ERROR:") {
		return "", fmt.Errorf("server error: %s", trimmed)
	}

	return trimmed, nil
}

// getEmbeddedFile reads a file from the embedded file system
func getEmbeddedFile(path string) ([]byte, error) {
	return embeddedFiles.ReadFile(path)
}

// getIconPath returns iconpath based on the OS
func getIconPath() string {
	switch os := runtime.GOOS; os {
	case "windows":
		_ = os 
		return "assets/wazuh-logo.ico"
	default:
		return "assets/wazuh-logo.png"
	}
}

// onExit is called when the application is terminated
func onExit() {
	log.Println("Frontend application stopped")
}