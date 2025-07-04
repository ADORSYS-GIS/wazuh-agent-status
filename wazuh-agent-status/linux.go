//go:build linux
// +build linux

package main

import (
	"fmt"
	"log"
	"os/exec"
	"strings"
	"time"
)

// checkServiceStatus checks the status of Wazuh agent and its connection on Linux
func checkServiceStatus() (string, string) {
	// Command to check agent status
	cmd := exec.Command("sh", "-c", "sudo /var/ossec/bin/wazuh-control status")
	output, err := cmd.Output()
	if err != nil {
		log.Printf("[%s] Error checking Wazuh agent status: %v\nOutput: %s", time.Now().Format(time.RFC3339), err, string(output))
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
	if connErr != nil {
		log.Printf("[%s] Error checking Wazuh agent connection: %v\nOutput: %s", time.Now().Format(time.RFC3339), connErr, string(connOutput))
	} else if strings.Contains(string(connOutput), "status='connected'") {
		connection = "Connected"
	}

	return status, connection
}

// restartAgent restarts the Wazuh agent on Linux
func restartAgent() {
	log.Printf("[%s] Restarting Wazuh agent...\n", time.Now().Format(time.RFC3339))
	cmd := exec.Command("sudo", "/var/ossec/bin/wazuh-control", "restart")
	err := cmd.Run()
	if err != nil {
		log.Printf("[%s] Failed to restart Wazuh agent: %v\n", time.Now().Format(time.RFC3339), err)
		if cmd.ProcessState != nil {
			log.Printf("[%s] Process state: %v\n", time.Now().Format(time.RFC3339), cmd.ProcessState)
		}
	} else {
		log.Printf("[%s] Wazuh agent restarted successfully\n", time.Now().Format(time.RFC3339))
	}
}

// updateAgent updates the Wazuh agent on Linux
func updateAgent() {
	log.Printf("[%s] Updating Wazuh agent...\n", time.Now().Format(time.RFC3339))
	cmd := exec.Command("sudo", "bash", "/var/ossec/active-response/bin/adorsys-update.sh")
	output, err := cmd.CombinedOutput()
	if err != nil {
		logFilePath := "/var/ossec/logs/active-responses.log"
		errorMessage := fmt.Sprintf("Update failed: %v. Output: %s. Check logs for details at %s", err, string(output), logFilePath)
		log.Printf("[%s] %s\n", time.Now().Format(time.RFC3339), errorMessage)
		if cmd.ProcessState != nil {
			log.Printf("[%s] Process state: %v\n", time.Now().Format(time.RFC3339), cmd.ProcessState)
		}
	} else {
		restartAgent()
		log.Printf("[%s] Wazuh agent updated successfully\n", time.Now().Format(time.RFC3339))
	}
}

func windowsMain() {
	// This function is intentionally left empty for Linux builds.
}