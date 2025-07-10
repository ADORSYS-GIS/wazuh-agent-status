package main

import (
	"bufio"
	"embed"
	"fmt"
	"log"
	"net"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"time"

	"github.com/getlantern/systray"
	"gopkg.in/natefinch/lumberjack.v2"
)

//go:embed assets/*
var embeddedFiles embed.FS

var (
	statusItem, connectionItem, updateItem, versionItem *systray.MenuItem
	enabledIcon, disabledIcon                           []byte
	isMonitoringUpdate                                  bool
)

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

// Set up log rotation using lumberjack.func init() {
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
	// Load main icon
	mainIcon, err := getEmbeddedFile(getIconPath())
	if err != nil {
		log.Fatalf("Failed to load main icon: %v", err)
	}
	systray.SetIcon(mainIcon)
	systray.SetTooltip("Wazuh Agent Status")

	// Load status icons.
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
	

	// Start background status update
	go monitorStatus()

	// Handle menu item clicks
	go handleMenuActions()
}

// monitorStatus continuously fetches and updates the agent status
func monitorStatus() {
	go func() {
		for {
			status, connection := fetchStatus()

			// Update status menu item
			if status == "Active" {
				statusItem.SetTitle("Agent: Active")
				statusItem.SetIcon(enabledIcon)
			} else {
				statusItem.SetTitle("Agent: Inactive")
				statusItem.SetIcon(disabledIcon)
			}

			// Update connection menu item
			if connection == "Connected" {
				connectionItem.SetTitle("Connection: Connected")
				connectionItem.SetIcon(enabledIcon)
			} else {
				connectionItem.SetTitle("Connection: Disconnected")
				connectionItem.SetIcon(disabledIcon)
			}

			if versionItem.String() == "v---" || versionItem.String() == "Version: Unknown" || versionItem.String() == "vUnknown" {
				checkVersionAfterUpdate()
			}

			time.Sleep(5 * time.Second)
		}
	}()

	go func() {
		for {
			checkVersion()

			time.Sleep(4 * time.Hour)
		}
	}()
}

func checkVersion() {
	versionStatus, version := fetchVersionStatus()

	if strings.HasPrefix(versionStatus, "Up to date") {
		versionItem.SetTitle(version)
		versionItem.Disable()
		updateItem.SetTitle("Up to date")
		updateItem.Disable()
	} else if strings.HasPrefix(versionStatus, "Outdated") {
		versionItem.SetTitle(version)
		versionItem.Disable()
		updateItem.SetTitle("Update")
		updateItem.Enable()
		log.Println("Version is outdated, starting update monitor...")
		startUpdateMonitor()
	} else {
		versionItem.SetTitle("Version: Unknown")
		versionItem.Disable()
		updateItem.Disable()
	}
}

func checkVersionAfterUpdate() {
	versionStatus, version := fetchVersionStatus()

	if strings.HasPrefix(versionStatus, "Up to date") {
		versionItem.SetTitle(version)
		versionItem.Disable()
		updateItem.SetTitle("Up to date")
		updateItem.Disable()
	} else if strings.HasPrefix(versionStatus, "Outdated") {
		versionItem.SetTitle(version)
		versionItem.Disable()
		updateItem.SetTitle("Update")
		updateItem.Enable()
	} else {
		versionItem.SetTitle("Version: Unknown")
		versionItem.Disable()
		updateItem.Disable()
	}
}

func fetchVersionStatus() (string, string) {
	conn, err := net.Dial("tcp", "localhost:50505")
	if err != nil {
		log.Printf("Failed to connect to backend: %v", err)
		return "Unknown", "Unknown"
	}
	defer conn.Close()

	fmt.Fprintln(conn, "check-version")
	reader := bufio.NewReader(conn)
	response, err := reader.ReadString('\n')
	if err != nil {
		log.Printf("Failed to read response: %v", err)
		return "Unknown", "Unknown"
	}

	response = strings.TrimSuffix(response, "\n")
	parts := strings.Split(response, ": ")
	if len(parts) < 2 {
		return "Unknown", "Unknown"
	}

	parts = strings.Split(parts[1], ", ")
	return parts[0], parts[1]
}

// startUpdateMonitor starts the update status monitoring if not already active
func startUpdateMonitor() {
	if isMonitoringUpdate {
		log.Println("Update monitoring is already running.")
		return
	}

	isMonitoringUpdate = true
	sendCommand("update")
	go monitorUpdateStatus()
}

// monitorUpdateStatus continuously fetches and updates the update status
func monitorUpdateStatus() {
	for isMonitoringUpdate {
		updateStatus := fetchUpdateStatus()

		// If the update status is "Disable", stop monitoring
		if updateStatus == "Disable" {
			log.Println("Update status is disabled. Stopping monitoring.")
			isMonitoringUpdate = false
			checkVersionAfterUpdate()
		} else {
			log.Printf("Current update status: %v", updateStatus)
			// Update the icon or text based on the update status
			updateItem.SetTitle("Updating...")
			updateItem.Disable()
		}

		// Sleep for a short period before checking again
		time.Sleep(5 * time.Second)
	}

	// Reset the flag after monitoring is done
	isMonitoringUpdate = false
}

// handleMenuActions listens for menu item clicks and performs actions
func handleMenuActions() {
	for range updateItem.ClickedCh {
		log.Println("Update clicked")
		startUpdateMonitor()
	}
}

// fetchStatus retrieves the agent status and connection state from the backend
func fetchStatus() (string, string) {
	conn, err := net.Dial("tcp", "localhost:50505") // Update the port if needed
	if err != nil {
		log.Printf("Failed to connect to backend: %v", err)
		return "Unknown", "Unknown"
	}
	defer conn.Close()

	// Request status
	fmt.Fprintln(conn, "status")

	// Use bufio.Reader to read the response (including newline characters)
	reader := bufio.NewReader(conn)
	response, err := reader.ReadString('\n') // Read until newline character
	if err != nil {
		log.Printf("Failed to read response: %v", err)
		return "Unknown", "Unknown"
	}

	response = strings.TrimSuffix(response, "\n")

	// Split the string by comma
	parts := strings.Split(response, ", ")

	// Extract the values
	status := strings.Split(parts[0], ": ")[1]
	connection := strings.Split(parts[1], ": ")[1]

	return status, connection
}

// fetchUpdateStatus retrieves the update status from the backend
func fetchUpdateStatus() string {
	conn, err := net.Dial("tcp", "localhost:50505") // Update the port if needed
	if err != nil {
		log.Printf("Failed to connect to backend: %v", err)
		return "Unknown"
	}
	defer conn.Close()

	// Send "update" command to the server
	fmt.Fprintln(conn, "update-status")

	// Use bufio.Reader to read the response (including newline characters)
	reader := bufio.NewReader(conn)
	response, err := reader.ReadString('\n') // Read until newline character
	if err != nil {
		log.Printf("Failed to read response: %v", err)
		return "Unknown"
	}

	// Trim the newline character
	response = strings.TrimSuffix(response, "\n")

	log.Printf("Update: %v", response)

	// Extract the value of the update status
	status := strings.Split(response, ": ")[1]
	return status
}

// sendCommand sends a command (e.g., pause or restart) to the backend
func sendCommand(command string) {
	conn, err := net.Dial("tcp", "localhost:50505") // Update the port if needed
	if err != nil {
		log.Printf("Failed to connect to backend: %v", err)
		return
	}
	defer conn.Close()

	// Send command
	fmt.Fprintln(conn, command)
}

// getEmbeddedFile reads a file from the embedded file system
func getEmbeddedFile(path string) ([]byte, error) {
	return embeddedFiles.ReadFile(path)
}

// getIconPath returns iconpath based on the OS
func getIconPath() string {
	switch os := runtime.GOOS; os {
	case "windows":
		return "assets/wazuh-logo.ico" // Path to the ICO icon for Windows
	default:
		return "assets/wazuh-logo.png" // Default icon path
	}
}

// onExit is called when the application is terminated
func onExit() {
	log.Println("Frontend application stopped")
}
