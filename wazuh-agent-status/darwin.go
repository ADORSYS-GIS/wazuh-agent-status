//go:build darwin
// +build darwin

package main

import (
	"log"
	"os/exec"
	"strings"
	"time"
	"fmt"
)

// checkServiceStatus checks the status of Wazuh agent and its connection on macOS
func checkServiceStatus() (string, string) {
	cmd := exec.Command("sh", "-c", "sudo /Library/Ossec/bin/wazuh-control status")
	output, err := cmd.Output()
	if err != nil {
		return "Inactive", "Disconnected"
	}

	status := "Inactive"
	if strings.Contains(string(output), "wazuh-agentd is running") {
		status = "Active"
	}

	// Check connection status
	connCmd := exec.Command("sh", "-c", "sudo grep ^status /Library/Ossec/var/run/wazuh-agentd.state")
	connOutput, connErr := connCmd.Output()
	connection := "Disconnected"
	if connErr == nil && strings.Contains(string(connOutput), "status='connected'") {
		connection = "Connected"
	}

	return status, connection
}

// restartAgent restarts the Wazuh agent on macOS
func restartAgent() {
	log.Printf("[%s] Restarting Wazuh agent...\n", time.Now().Format(time.RFC3339))
	err := exec.Command("sudo", "/Library/Ossec/bin/wazuh-control", "restart").Run()
	if err != nil {
		log.Printf("[%s] Failed to restart Wazuh agent: %v\n", time.Now().Format(time.RFC3339), err)
	} else {
		log.Printf("[%s] Wazuh agent restarted successfully\n", time.Now().Format(time.RFC3339))
	}
}

// updateAgent updates the Wazuh agent on macOS
func updateAgent() {
	logFilePath := "/Library/Ossec/logs/active-responses.log"
	log.Printf("[%s] Updating Wazuh agent...\n", time.Now().Format(time.RFC3339))
	err := exec.Command("sudo", "/Library/Ossec/active-response/bin/adorsys-update.sh").Run()
	if err != nil {
		errorMessage := fmt.Sprintf("Update failed: %v. Check logs: %s", err, logFilePath)
		log.Printf("[%s] %s\n", time.Now().Format(time.RFC3339), errorMessage)
	} else {
		restartAgent()
		log.Printf("[%s] Wazuh agent updated successfully\n", time.Now().Format(time.RFC3339))
	}
}

func windowsMain() {
	// This function is intentionally left empty for macOS builds.
}