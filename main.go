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

	// Start the monitoring function
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

// monitorStatusAndConnection checks the status and connection every 10 seconds and updates the tray items accordingly
func monitorStatusAndConnection(connectedIcon, disconnectedIcon []byte) {
	for {
		status, connection := checkServiceStatus()

		if status == "Active" {
			statusItem.SetTitle(fmt.Sprintf("Status: %s", status))
			statusItem.SetIcon(connectedIcon)
		} else {
			statusItem.SetTitle(fmt.Sprintf("Status: %s", status))
			statusItem.SetIcon(disconnectedIcon)
		}

		if connection == "Connected" {
			connectionItem.SetTitle(fmt.Sprintf("Connection: %s", connection))
			connectionItem.SetIcon(connectedIcon)
		} else {
			connectionItem.SetTitle(fmt.Sprintf("Connection: %s", connection))
			connectionItem.SetIcon(disconnectedIcon)
		}

		time.Sleep(10 * time.Second) // Update every 10 seconds
	}
}

// checkServiceStatus checks the status and connection of the Wazuh agent based on the OS
func checkServiceStatus() (string, string) {
	const (
		linuxStatusCmd       = "sudo systemctl status wazuh-agent.service"
		linuxConnectionCmd   = "sudo grep ^status /var/ossec/var/run/wazuh-agentd.state"
		darwinStatusCmd      = "sudo /Library/Ossec/bin/wazuh-control status"
		darwinConnectionCmd  = "sudo grep ^status /Library/Ossec/var/run/wazuh-agentd.state"
		windowsStatusCmd     = `C:\Program Files (x86)\ossec\bin\wazuh-control status`
		windowsConnectionCmd = `powershell -Command "Select-String -Path 'C:\Program Files (x86)\ossec-agent\wazuh-agent.state' -Pattern '^status'"`
	)

	var statusCmd, connectionCmd *exec.Cmd
	switch runtime.GOOS {
	case "linux":
		statusCmd = exec.Command("sh", "-c", linuxStatusCmd)
		connectionCmd = exec.Command("sh", "-c", linuxConnectionCmd)
	case "darwin":
		statusCmd = exec.Command("sh", "-c", darwinStatusCmd)
		connectionCmd = exec.Command("sh", "-c", darwinConnectionCmd)
	case "windows":
		statusCmd = exec.Command("cmd", "/C", windowsStatusCmd)
		connectionCmd = exec.Command("cmd", "/C", windowsConnectionCmd)
	default:
		return "Unsupported OS", "Unsupported OS"
	}

	status := getStatus(statusCmd)
	connection := getConnection(connectionCmd)

	return status, connection
}

func getStatus(cmd *exec.Cmd) string {
	output, err := cmd.Output()
	if err != nil {
		return "Inactive"
	}

	stdout := string(output)
	if runtime.GOOS == "linux" && strings.Contains(stdout, "Active: active (running)") {
		return "Active"
	}

	for _, line := range strings.Split(stdout, "\n") {
		if strings.Contains(line, "is running...") {
			return "Active"
		}
	}

	return "Inactive"
}

func getConnection(cmd *exec.Cmd) string {
	output, err := cmd.Output()
	if err != nil {
		return "Disconnected"
	}

	if strings.Contains(string(output), "status='connected'") {
		return "Connected"
	}

	return "Disconnected"
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
