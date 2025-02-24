package main

import (
	"bufio"
	"embed"
	"fmt"
	"log"
	"net"
	"runtime"
	"strings"
	"time"
	"os"
	"path/filepath"

	"github.com/getlantern/systray"
	"gopkg.in/natefinch/lumberjack.v2"
)

//go:embed assets/*
var embeddedFiles embed.FS

var (
	statusItem, connectionItem, pauseItem, updateItem, restartItem, versionItem *systray.MenuItem
	enabledIcon, disabledIcon                                                   []byte
	isMonitoringUpdate                                                          bool
)

// Set up log rotation using lumberjack.func init() {
	func init() {
		homeDir, err := os.UserHomeDir()
		if err != nil {
			log.Fatalf("failed to get home directory: %v", err)
		}
	
		logFilePath := filepath.Join(homeDir, ".wazuh-agent-status-client.log")
	
		log.SetOutput(&lumberjack.Logger{
			Filename:   logFilePath,
			MaxSize:    10,          
			MaxBackups: 3,           
			MaxAge:     28,          
			Compress:   true,       
		})
	}

// Main entry point
func main() {
	log.Println("Starting frontend...")
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
	connectionItem = systray.AddMenuItem("Connection: Unknown", "Wazuh Agent Connection")
	systray.AddSeparator()
	pauseItem = systray.AddMenuItem("Pause", "Pause the Wazuh Agent")
	restartItem = systray.AddMenuItem("Restart", "Restart the Wazuh Agent")
	updateItem = systray.AddMenuItem("Update", "Update the Wazuh Agent")
	quitItem := systray.AddMenuItem("Quit", "Quit the application")
	systray.AddSeparator()
	versionItem = systray.AddMenuItem("Up to date", "The version state of the wazuhbsetup")

	// Start background status update
	go monitorStatus()

	// Handle menu item clicks
	go handleMenuActions(quitItem)
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

			time.Sleep(5 * time.Second)
		}
	}()

	go func() {
		for {
			checkVersion()

			time.Sleep(1 * time.Hour)
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
	// Only start monitoring if it's not already active
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
			checkVersion()
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
func handleMenuActions(quitItem *systray.MenuItem) {
	for {
		select {
		case <-pauseItem.ClickedCh:
			log.Println("Pause clicked")
			sendCommand("pause")
		case <-restartItem.ClickedCh:
			log.Println("Restart clicked")
			sendCommand("restart")
		case <-updateItem.ClickedCh:
			log.Println("Update clicked")
			startUpdateMonitor()
		case <-quitItem.ClickedCh:
			log.Println("Quit clicked")
			systray.Quit()
		}
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
