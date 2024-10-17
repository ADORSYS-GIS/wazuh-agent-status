package main

import (
	"embed"
	"fmt"
	"log"
	"time"

	"github.com/getlantern/systray"
)

//go:embed assets/*
var embeddedFiles embed.FS // Embeds all files in the assets folder

var statusItem, connectionItem, pauseItem, restartItem *systray.MenuItem

var statusTitle string

var connectionTitle string

func main() {
	log.Println("Wazuh agent status started...")
	systray.Run(onReady, onExit)
}

func onReady() {
	// Set the main icon
	iconData, err := getEmbeddedFile(getIconPath())
	if err != nil {
		log.Fatalf("Failed to load icon: %v", err)
	}
	systray.SetIcon(iconData)
	systray.SetTooltip("Wazuh Agent Status")

	// Load icons for status and connection
	connectedIcon, _ := getEmbeddedFile("assets/green-dot.png")
	disconnectedIcon, _ := getEmbeddedFile("assets/gray-dot.png")

	// Load icons for pause and restart
	pauseIconData, err := getEmbeddedFile("assets/pause.png")
	if err != nil {
		log.Printf("Failed to load pause icon: %v", err)
	}
	restartIconData, err := getEmbeddedFile("assets/restart.png")
	if err != nil {
		log.Printf("Failed to load restart icon: %v", err)
	}

	// Set the menu items
	statusItem = systray.AddMenuItem("Agent: Inactive", "Wazuh Agent Status")
	connectionItem = systray.AddMenuItem("Connection: Disconnected", "Wazuh Agent Connection")
	systray.AddSeparator()

	// Add pause and restart items with icons
	pauseItem = systray.AddMenuItem("Pause", "Pause the Wazuh Agent")
	if pauseIconData != nil {
		pauseItem.SetIcon(pauseIconData)
	}

	restartItem = systray.AddMenuItem("Restart", "Restart the Wazuh Agent")
	if restartIconData != nil {
		restartItem.SetIcon(restartIconData)
	}

	quitItem := systray.AddMenuItem("Quit", "Quit the Agent application")

	// Start background monitoring
	go monitorStatusAndConnection(connectedIcon, disconnectedIcon)

	go func() {
		for {
			select {
			case <-pauseItem.ClickedCh:
				pauseAgent()
			case <-restartItem.ClickedCh:
				restartAgent()
			case <-quitItem.ClickedCh:
				systray.Quit()
			}
		}
	}()
}

func onExit() {
	log.Println("Wazuh agent status stopped")
}

// monitorStatusAndConnection periodically checks the status of the Wazuh agent
func monitorStatusAndConnection(connectedIcon, disconnectedIcon []byte) {
	for {
		status, connection := checkServiceStatus()

		if status == "Active" {
			if statusTitle != "Agent: Active" {
				log.Printf("[%s] Wazuh agent is now active\n", time.Now().Format(time.RFC3339))
				statusTitle = "Agent: Active"
			}
			statusItem.SetTitle(statusTitle)
			statusItem.SetIcon(connectedIcon)
		} else {
			if statusTitle != "Agent: Inactive" {
				log.Printf("[%s] Wazuh agent is now inactive\n", time.Now().Format(time.RFC3339))
				statusTitle = "Agent: Inactive"
			}
			statusItem.SetTitle(statusTitle)
			statusItem.SetIcon(disconnectedIcon)
		}

		if connection == "Connected" {
			if connectionTitle != "Connection: Connected" {
				log.Printf("[%s] Wazuh agent connection established\n", time.Now().Format(time.RFC3339))
				connectionTitle = "Connection: Connected"
			}
			connectionItem.SetTitle(connectionTitle)
			connectionItem.SetIcon(connectedIcon)
		} else {
			if connectionTitle != "Connection: Disconnected" {
				log.Printf("[%s] Wazuh agent connection lost\n", time.Now().Format(time.RFC3339))
				connectionTitle = "Connection: Disconnected"
			}
			connectionItem.SetTitle(connectionTitle)
			connectionItem.SetIcon(disconnectedIcon)
		}

		time.Sleep(5 * time.Second)
	}
}

// getEmbeddedFile reads a file from the embedded file system
func getEmbeddedFile(path string) ([]byte, error) {
	fileData, err := embeddedFiles.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read embedded file: %w", err)
	}
	return fileData, nil
}
