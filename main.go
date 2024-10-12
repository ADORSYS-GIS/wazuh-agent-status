package main

import (
	"embed"
	"fmt"
	"log"
	"os/exec"
	"runtime"
	"strings"
	"time"

	"github.com/getlantern/systray"
)

//go:embed assets/*
var embeddedFiles embed.FS // Embeds all files in the assets folder

var statusItem, connectionItem, pauseItem, restartItem *systray.MenuItem

func main() {
	log.Println("Wazuh agent status started...")
	systray.Run(onReady, onExit)
}

func onReady() {
	var iconPath string
	switch runtime.GOOS {
	case "linux", "darwin":
		iconPath = "assets/wazuh-logo.png"
	case "windows":
		iconPath = "assets/wazuh-logo-min.ico"
	}

	// Get the icon from the embedded file system
	iconData, err := getEmbeddedFile(iconPath)
	if err != nil {
		log.Fatalf("Failed to load icon: %v", err)
	}

	systray.SetIcon(iconData)
	systray.SetTooltip("Wazuh Agent Status")

	// Load icons for the status and connection
	connectedIcon, _ := getEmbeddedFile("assets/green-dot.png")
	disconnectedIcon, _ := getEmbeddedFile("assets/gray-dot.png")

	statusItem = systray.AddMenuItem("Agent Status: Checking...", "Wazuh Agent Status")
	connectionItem = systray.AddMenuItem("Connection Status: Checking...", "Wazuh Agent Connection")
	systray.AddSeparator()

	// Load icons for the pause and restart items
	pauseIconData, _ := getEmbeddedFile("assets/pause.png")
	restartIconData, _ := getEmbeddedFile("assets/restart.png")

	// Add menu items with icons
	pauseItem = systray.AddMenuItem("Pause", "Pause the Wazuh Agent")
	pauseItem.SetIcon(pauseIconData)
	restartItem = systray.AddMenuItem("Restart", "Restart the Wazuh Agent")
	restartItem.SetIcon(restartIconData)

	quitItem := systray.AddMenuItem("Quit", "Quit the Agent application")

	go func() {
		for {
			updateStatus(connectedIcon, disconnectedIcon)
			time.Sleep(10 * time.Second) // Update every 10 seconds
		}
	}()

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

// updateStatus updates the agent's status and connection status in the tray
func updateStatus(connectedIcon, disconnectedIcon []byte) {
	status, connection := checkServiceStatus()

	if status == "Active" {
		statusItem.SetTitle(fmt.Sprintf("Status: %s", status))
		statusItem.SetIcon(connectedIcon)
	} else {
		statusItem.SetTitle(fmt.Sprintf("Status: %s", status))
		statusItem.SetIcon(disconnectedIcon)
		go monitorStatusAndConnection(connectedIcon, disconnectedIcon)
	}

	if connection == "Connected" {
		connectionItem.SetTitle(fmt.Sprintf("Connection: %s", connection))
		connectionItem.SetIcon(connectedIcon)
	} else {
		connectionItem.SetTitle(fmt.Sprintf("Connection: %s", connection))
		connectionItem.SetIcon(disconnectedIcon)
		go monitorStatusAndConnection(connectedIcon, disconnectedIcon)
	}
}

// monitorStatusAndConnection checks the status and connection every 10 seconds if they are not in their desired states
func monitorStatusAndConnection(connectedIcon, disconnectedIcon []byte) {
	for {
		time.Sleep(10 * time.Second)
		status, connection := checkServiceStatus()

		if status == "Active" {
			statusItem.SetIcon(connectedIcon)
			statusItem.SetTitle("Status: Active")
		} else {
			statusItem.SetIcon(disconnectedIcon)
			statusItem.SetTitle("Status: Rechecking...")
		}

		if connection == "Connected" {
			connectionItem.SetIcon(connectedIcon)
			connectionItem.SetTitle("Connection: Connected")
		} else {
			connectionItem.SetIcon(disconnectedIcon)
			connectionItem.SetTitle("Connection: Rechecking...")
		}

		// Stop monitoring when both are resolved
		if status == "Active" && connection == "Connected" {
			break
		}
	}
}

// checkServiceStatus checks the status and connection of the Wazuh agent
func checkServiceStatus() (string, string) {
	var statusCmd, connectionCmd *exec.Cmd
	switch runtime.GOOS {
	case "linux":
		statusCmd = exec.Command("sudo", "systemctl", "status", "wazuh-agent.service")
		connectionCmd = exec.Command("sudo", "grep", "^status", "/var/ossec/var/run/wazuh-agentd.state")
	case "darwin":
		statusCmd = exec.Command("sudo", "/Library/Ossec/bin/wazuh-control", "status")
		connectionCmd = exec.Command("sudo", "grep", "^status", "/Library/Ossec/var/run/wazuh-agentd.state")
	case "windows":
		statusCmd = exec.Command("C:\\Program Files (x86)\\ossec\\bin\\wazuh-control", "status")
		connectionCmd = exec.Command("powershell", "-Command", "Select-String -Path 'C:\\Program Files (x86)\\ossec-agent\\wazuh-agent.state' -Pattern '^status'")
	default:
		return "Unsupported OS", "Unsupported OS"
	}

	statusOutput, statusErr := statusCmd.Output()
	connectionOutput, connectionErr := connectionCmd.Output()

	status := "Inactive"
	connection := "Disconnected"

	if statusErr == nil {
		stdout := string(statusOutput)
		if runtime.GOOS == "linux" {
			if strings.Contains(stdout, "Active: active (running)") {
				status = "Active"
			}
		} else {
			for _, line := range strings.Split(stdout, "\n") {
				if strings.Contains(line, "is running...") {
					status = "Active"
					break
				}
			}
		}
	}

	if connectionErr == nil {
		stdout := string(connectionOutput)
		if strings.Contains(stdout, "status='connected'") {
			connection = "Connected"
		}
	}

	return status, connection
}

// pauseAgent pauses the Wazuh agent based on the OS
func pauseAgent() {
	var cmd *exec.Cmd
	switch runtime.GOOS {
	case "linux":
		cmd = exec.Command("sudo", "systemctl", "stop", "wazuh-agent.service")
	case "darwin":
		cmd = exec.Command("sudo", "/Library/Ossec/bin/wazuh-control", "stop")
	case "windows":
		cmd = exec.Command("net", "stop", "wazuh-agent")
	default:
		log.Println("Unsupported OS for pausing the agent")
		return
	}

	if err := cmd.Run(); err != nil {
		log.Printf("Failed to pause agent: %v", err)
	} else {
		log.Println("Wazuh agent paused successfully")
	}
}

// restartAgent restarts the Wazuh agent based on the OS
func restartAgent() {
	var cmd *exec.Cmd
	switch runtime.GOOS {
	case "linux":
		cmd = exec.Command("sudo", "systemctl", "restart", "wazuh-agent.service")
	case "darwin":
		cmd = exec.Command("sudo", "/Library/Ossec/bin/wazuh-control", "restart")
	case "windows":
		cmd = exec.Command("net", "start", "wazuh-agent")
	default:
		log.Println("Unsupported OS for restarting the agent")
		return
	}

	if err := cmd.Run(); err != nil {
		log.Printf("Failed to restart agent: %v", err)
	} else {
		log.Println("Wazuh agent restarted successfully")
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
