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

var statusItem, connectionItem *systray.MenuItem

func main() {
	log.Println("Wazuh agent status started...")
	systray.Run(onReady, onExit)
}

func onReady() {
	var iconPath string
	switch runtime.GOOS {
	case "linux", "darwin":
		iconPath = "assets/wazuh-logo-min.png"
	case "windows":
		iconPath = "assets/wazuh-logo-min.ico"
	}

	// Get the icon from the embedded file system
	iconData, err := getEmbeddedFile(iconPath)
	if err != nil {
		log.Fatalf("Failed to load icon: %v", err)
	}

	systray.SetIcon(iconData)
	systray.SetTitle("Wazuh Agent")
	systray.SetTooltip("Wazuh Agent Status")

	statusItem = systray.AddMenuItem("Status: Checking...", "Wazuh Agent Status")
	connectionItem = systray.AddMenuItem("Connection: Checking...", "Wazuh Agent Connection")

	quitItem := systray.AddMenuItem("Quit", "Quit the application")

	go func() {
		for {
			updateStatus()
			time.Sleep(5 * time.Second)
		}
	}()

	go func() {
		<-quitItem.ClickedCh
		systray.Quit()
	}()
}

func onExit() {
	log.Println("Wazuh agent status stopped")
}

func updateStatus() {
	status, connection := checkServiceStatus()
	statusItem.SetTitle(fmt.Sprintf("Status: %s", status))
	connectionItem.SetTitle(fmt.Sprintf("Connection: %s", connection))
}

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

func getEmbeddedFile(path string) ([]byte, error) {
	// Read the file from the embedded file system
	fileData, err := embeddedFiles.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read embedded file: %w", err)
	}
	return fileData, nil
}
