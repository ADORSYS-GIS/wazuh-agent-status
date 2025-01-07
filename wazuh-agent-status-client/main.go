package main

import (
	"embed"
	"fmt"
	"log"
	"net"
	"time"
	"strings"
	"bufio"
	"runtime"

	"github.com/getlantern/systray"
)

//go:embed assets/*
var embeddedFiles embed.FS

var (
	statusItem, connectionItem, updateItem, syncItem *systray.MenuItem
	statusIconConnected, statusIconDisconnected       []byte
	connectionIconConnected, connectionIconDisconnected []byte
)

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

	// Load icons for status and connection
	statusIconConnected, _ = getEmbeddedFile("assets/green-dot.png")
	statusIconDisconnected, _ = getEmbeddedFile("assets/gray-dot.png")
	connectionIconConnected, _ = getEmbeddedFile("assets/green-dot.png")
	connectionIconDisconnected, _ = getEmbeddedFile("assets/gray-dot.png")

	// Create menu items
	statusItem = systray.AddMenuItem("Agent: Unknown", "Wazuh Agent Status")
	connectionItem = systray.AddMenuItem("Connection: Unknown", "Wazuh Agent Connection")
	systray.AddSeparator()
	updateItem = systray.AddMenuItem("Update", "Update the Wazuh Agent")
	syncItem = systray.AddMenuItem("Sync", "Sync the Wazuh Agent")

	// Start background status update
	go monitorStatus()
	
	// Handle menu item clicks
	go handleMenuActions()
}

// monitorStatus continuously fetches and updates the agent status
func monitorStatus() {
	for {
		status, connection := fetchStatus()

		// Update status menu item
		if status == "Active" {
			statusItem.SetTitle("Agent: Active")
			statusItem.SetIcon(statusIconConnected)
		} else {
			statusItem.SetTitle("Agent: Inactive")
			statusItem.SetIcon(statusIconDisconnected)
		}

		// Update connection menu item
		if connection == "Connected" {
			connectionItem.SetTitle("Connection: Connected")
			connectionItem.SetIcon(connectionIconConnected)
		} else {
			connectionItem.SetTitle("Connection: Disconnected")
			connectionItem.SetIcon(connectionIconDisconnected)
		}

		time.Sleep(5 * time.Second)
	}
}

// handleMenuActions listens for menu item clicks and performs actions
func handleMenuActions() {
	for {
		select {
			case <-updateItem.ClickedCh:
				log.Println("Update clicked")
				sendCommand("update")
			case <-syncItem.ClickedCh:
				log.Println("Sync clicked")
				sendCommand("sync")
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
	
	// Log the raw response for debugging
	log.Printf("Raw response: %s", response)
	
	// Split the string by comma
    parts := strings.Split(response, ", ")

    // Extract the values
    status := strings.Split(parts[0], ": ")[1]
    connection := strings.Split(parts[1], ": ")[1]
	
	return status, connection
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
