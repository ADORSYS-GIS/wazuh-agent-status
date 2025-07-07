//go:build darwin
// +build darwin

package main

import (
	"fmt"
	"log"
	"os/exec"
	"strings"
)

const (
	sudoCommand = "/bin/sudo"
)

// checkServiceStatus checks the status of Wazuh agent and its connection on macOS
func checkServiceStatus() (string, string) {
	cmd := exec.Command(sudoCommand, "/Library/Ossec/bin/wazuh-control", "status")
	output, err := cmd.CombinedOutput()
	if err != nil {
		return "Inactive", "Disconnected"
	}

	status := "Inactive"
	if strings.Contains(string(output), "wazuh-agentd is running") {
		status = "Active"
	}

	// Check connection status
	connCmd := exec.Command(sudoCommand, "grep ^status /Library/Ossec/var/run/wazuh-agentd.state")
	connOutput, connErr := connCmd.CombinedOutput()
	connection := "Disconnected"
	if connErr == nil && strings.Contains(string(connOutput), "status='connected'") {
		connection = "Connected"
	}

	return status, connection
}

// restartAgent restarts the Wazuh agent on macOS
func restartAgent() {
	log.Printf("Restarting Wazuh agent...\n")
	cmd := exec.Command(sudoCommand, "/Library/Ossec/bin/wazuh-control", "restart")
	output, err := cmd.CombinedOutput()
	if err != nil {
		log.Printf("Failed to restart Wazuh agent: %v\n", string(output))
	} else {
		log.Printf("Wazuh agent restarted successfully\n")
	}
}

// updateAgent updates the Wazuh agent on macOS
func updateAgent() {
	logFilePath := "/Library/Ossec/logs/active-responses.log"
	log.Printf("Updating Wazuh agent...\n")
	cmd := exec.Command(sudoCommand, "/Library/Ossec/active-response/bin/adorsys-update.sh")
	output, err := cmd.CombinedOutput()
	if err != nil {
		errorMessage := fmt.Sprintf("Update failed: %v. Check logs: %s", string(output), logFilePath)
		log.Printf("%s\n", errorMessage)
	} else {
		restartAgent()
		log.Printf("Wazuh agent updated successfully\n")
	}
}

func windowsMain() {
	// This function is intentionally left empty for macOS builds.
}