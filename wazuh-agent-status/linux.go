//go:build linux
// +build linux

package main

import (
	"log"
	"os"
	"os/exec"
	"strings"
	"time"
	"fmt"
)

// checkServiceStatus checks the status of Wazuh agent and its connection on Linux
func checkServiceStatus() (string, string) {
	// Command to check agent status
	cmd := exec.Command("sh", "-c", "sudo /var/ossec/bin/wazuh-control status")
	output, err := cmd.Output()
	if err != nil {
		return "Inactive", "Disconnected"
	}

	status := "Inactive"
	if strings.Contains(string(output), "wazuh-agentd is running") {
		status = "Active"
	}

	// Check connection status
	connCmd := exec.Command("sh", "-c", "sudo grep ^status /var/ossec/var/run/wazuh-agentd.state")
	connOutput, connErr := connCmd.Output()
	connection := "Disconnected"
	if connErr == nil && strings.Contains(string(connOutput), "status='connected'") {
		connection = "Connected"
	}

	return status, connection
}

// pauseAgent pauses the Wazuh agent on Linux
func pauseAgent() {
	log.Printf("[%s] Pausing Wazuh agent...\n", time.Now().Format(time.RFC3339))
	err := exec.Command("sudo", "/var/ossec/bin/wazuh-control", "stop").Run()
	if err != nil {
		log.Printf("[%s] Failed to pause Wazuh agent: %v\n", time.Now().Format(time.RFC3339), err)
	} else {
		log.Printf("[%s] Wazuh agent paused successfully\n", time.Now().Format(time.RFC3339))
	}
}

// restartAgent restarts the Wazuh agent on Linux
func restartAgent() {
	log.Printf("[%s] Restarting Wazuh agent...\n", time.Now().Format(time.RFC3339))
	err := exec.Command("sudo", "/var/ossec/bin/wazuh-control", "restart").Run()
	if err != nil {
		log.Printf("[%s] Failed to restart Wazuh agent: %v\n", time.Now().Format(time.RFC3339), err)
	} else {
		log.Printf("[%s] Wazuh agent restarted successfully\n", time.Now().Format(time.RFC3339))
	}
}

// Function to check if a path exists
func pathExists(path string) bool {
    _, err := os.Stat(path)
    return !os.IsNotExist(err)
}

func notifyUser(title, message string) {
	iconPath := "/usr/share/pixmaps/wazuh-logo.png"
	if pathExists(iconPath) {
		exec.Command("notify-send", "--app-name=Wazuh", "-u", "critical", title, message, "-i", iconPath).Run()
	} else {
		exec.Command("notify-send", "--app-name=Wazuh", "-u", "critical", title, message).Run()
	}
}

// updateAgent updates the Wazuh agent on Linux
func updateAgent() {
	log.Printf("[%s] Updating Wazuh agent...\n", time.Now().Format(time.RFC3339))
	err := exec.Command("sudo", "bash", "/var/ossec/active-response/bin/dorsys-update.sh").Run()
	if err != nil {
		logFilePath := "/var/ossec/logs/active-responses.log"
		errorMessage := fmt.Sprintf("Update failed: Check logs for details at %s", logFilePath)
		log.Printf("[%s] %s\n", time.Now().Format(time.RFC3339), errorMessage)
		notifyUser("Wazuh Agent Update", errorMessage)
	} else {
		restartAgent()
		log.Printf("[%s] Wazuh agent updated successfully\n", time.Now().Format(time.RFC3339))
		notifyUser("Wazuh Agent Update", "Update successful!")
	}
}

func windowsMain() {

}