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
	"regexp"

	"fyne.io/systray" 
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

var titleRegexp = regexp.MustCompile(`"(.*?)"`)

// Version is set at build time via ldflags
var Version = "dev"

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
		go monitorVersion()
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

// monitorStatusStream establishes a persistent connection to receive status updates. (UNCHANGED)
func monitorStatusStream() {
	for {
		conn, err := net.DialTimeout("tcp", "localhost:50505", 5*time.Second)
		if err != nil {
			log.Printf("Failed to connect to backend for status stream: %v. Retrying in 5s...", err)
			updateStatusItems("Unknown", "Unknown")
			time.Sleep(5 * time.Second)
			continue
		}

		log.Println("Status stream connected. Sending subscription...")
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

// updateStatusItems safely updates the systray menu items. (UNCHANGED)
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

// monitorVersion handles the startup check and the periodic check. (UNCHANGED)
func monitorVersion() {
	// Loop indefinitely. The inner logic handles breaking out into the long poll.
	for {
		// Attempt to get the version.
		handleVersionCheck(true)

		// Check if the version is still in its initial or an error state.
		// If so, wait 5 seconds and try again.
		currentVersion := getMenuItemTitle(versionItem.String())
		log.Println("Current Version:", currentVersion)
		if currentVersion == "" || currentVersion == "---" || currentVersion == "v---" || currentVersion == "Unknown" {
			log.Println("Version is in default/error state, retrying in 5 seconds...")
			time.Sleep(5 * time.Second)
		} else {
			// If a valid version is fetched, break out of the initial retry loop.
			log.Println("Version check successful. Switching to 4-hour polling interval.")
			break
		}
	}

	// Once the initial version is fetched, switch to long polling.
	for {
		time.Sleep(4 * time.Hour)
		handleVersionCheck(true)
	}
}

// handleVersionCheck communicates with the backend, updates the menu, and conditionally starts an update. (UNCHANGED)
func handleVersionCheck(autoStart bool) {
	response, err := sendCommandAndReceive("get-version")
	if err != nil {
		log.Printf("Error fetching version: %v", err)
		versionItem.SetTitle("Unknown")
		updateItem.SetTitle("---")
		updateItem.Disable()
		return
	}

	response = strings.TrimPrefix(response, "VERSION_CHECK: ")
	isOutdated := strings.Contains(response, "Outdated")
	
	// --- Update Menu Items ---
	if isOutdated {
		parts := strings.Split(response, ", ")
		version := "Unknown"
		if len(parts) == 2 {
			version = parts[1]
		}
		versionItem.SetTitle(version)
		updateItem.SetTitle("Update Available")
		updateItem.Enable()
		
		if autoStart {
			log.Println("Automatic update triggered by periodic check.")
			updateItem.Disable()
			go startUpdateStream()
		}
		
	} else if strings.Contains(response, "Up to date") {
		parts := strings.Split(response, ", ")
		version := "Unknown"
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
		go startUpdateStream() 
	}
}

// startUpdateStream initiates the update on the server and streams the progress back. (FIXED LOGIC - UNCHANGED)
func startUpdateStream() {
	// Use mutex to prevent multiple update streams from starting simultaneously
	if !updateMutex.TryLock() {
		log.Println("Update already in progress. Skipping.")
		return
	}
	defer updateMutex.Unlock()

	conn, err := net.DialTimeout("tcp", "localhost:50505", 5*time.Second)
	if err != nil {
		log.Printf("Failed to connect to backend for update stream: %v", err)
		updateItem.SetTitle("Update Failed (Connect)")
		updateItem.Enable() 
		return
	}
	defer conn.Close()

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
				// Set a temporary status before the synchronous check
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
	
	// After the stream ends, perform a synchronous version check to update the menu accurately
	handleVersionCheck(false) 
}

// sendCommandAndReceive is a general utility for single request/response commands. (UNCHANGED)
func sendCommandAndReceive(command string) (string, error) {
	conn, err := net.DialTimeout("tcp", "localhost:50505", 3*time.Second)
	if err != nil {
		return "", fmt.Errorf("failed to dial: %w", err)
	}
	defer conn.Close()

	_ = conn.SetReadDeadline(time.Now().Add(5 * time.Second))

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

// getEmbeddedFile reads a file from the embedded file system (UNCHANGED)
func getEmbeddedFile(path string) ([]byte, error) {
	return embeddedFiles.ReadFile(path)
}

// getIconPath returns iconpath based on the OS (UNCHANGED)
func getIconPath() string {
	switch os := runtime.GOOS; os {
	case "windows":
		_ = os 
		return "assets/wazuh-logo.ico"
	default:
		return "assets/wazuh-logo.png"
	}
}

// getMenuItemTitle extracts the Title from MenuItem.String()
// formatted like "MenuItem[N, "Title"]".
func getMenuItemTitle(input string) (string) {
	// Find the text between the quotes
	matches := titleRegexp.FindStringSubmatch(input)
	if len(matches) > 1 {
		// matches[1] is the content between the quotes
		return matches[1]
	}
	return ""
}

// onExit is called when the application is terminated (UNCHANGED)
func onExit() {
	log.Println("Frontend application stopped")
}