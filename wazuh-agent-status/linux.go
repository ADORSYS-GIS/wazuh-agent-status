//go:build linux
// +build linux

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


// checkServiceStatus checks the status of Wazuh agent and its connection on Linux
func checkServiceStatus() (string, string) {
	// Command to check agent status
	cmd := exec.Command(sudoCommand, "/var/ossec/bin/wazuh-control", "status")
	output, err := cmd.CombinedOutput()
	if err != nil {
		log.Printf("Error checking Wazuh agent status: %v\nOutput: %s", err, string(output))
		return "Inactive", "Disconnected"
	}

	status := "Inactive"
	if strings.Contains(string(output), "wazuh-agentd is running") {
		status = "Active"
	}

	// Check connection status
	connCmd := exec.Command(sudoCommand, "grep ^status /var/ossec/var/run/wazuh-agentd.state")
	connOutput, connErr := connCmd.CombinedOutput()
	connection := "Disconnected"
	if connErr != nil {
		log.Printf("Error checking Wazuh agent connection: %v\nOutput: %s", connErr, string(connOutput))
	} else if strings.Contains(string(connOutput), "status='connected'") {
		connection = "Connected"
	}

	return status, connection
}

// updateAgent updates the Wazuh agent on Linux
func updateAgent() {
	log.Printf("Updating Wazuh agent...\n")
	cmd := exec.Command(sudoCommand, "/var/ossec/active-response/bin/adorsys-update.sh")
	output, err := cmd.CombinedOutput()
	if err != nil {
		logFilePath := "/var/ossec/logs/active-responses.log"
		errorMessage := fmt.Sprintf("Update failed: %v. Check logs for details at %s", string(output), logFilePath)
		log.Printf("%s\n", errorMessage)
	} else {
		log.Printf("Wazuh agent updated successfully\n")
	}
}

func windowsMain() {
	// This function is intentionally left empty for Linux builds.
}