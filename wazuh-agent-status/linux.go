//go:build linux
// +build linux

package main

import (
	"os/exec"
	"strings"
	"log"
	"time"
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

// updateAgent updates the Wazuh agent on Linux
func updateAgent() {
	log.Printf("[%s] Updating Wazuh agent...\n", time.Now().Format(time.RFC3339))
	err := exec.Command("sudo", "/var/ossec/active-response/bin/adorsys-update.sh").Run()
	if err != nil {
		log.Printf("[%s] Failed to update the Wazuh agent: %v\n", time.Now().Format(time.RFC3339), err)
	} else {
		log.Printf("[%s] Wazuh agent updated successfully\n", time.Now().Format(time.RFC3339))
	}
}

func windowsMain() {

}