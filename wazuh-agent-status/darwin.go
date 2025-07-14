//go:build darwin
// +build darwin

package main

import (
	"fmt"
	"log"
	"os/exec"
	"strings"
	"time"
)

const (
	sudoCommand      = "sudo"
	wazuhControlPath = "/Library/Ossec/bin/wazuh-control"
)

// checkServiceStatus checks the status of Wazuh agent and its connection on macOS
func checkServiceStatus() (string, string) {
	cmd := exec.Command(sudoCommand, wazuhControlPath, "status")
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

// pauseAgent pauses the Wazuh agent on macOS
func pauseAgent() {
	log.Printf("[%s] Pausing Wazuh agent...\n", time.Now().Format(time.RFC3339))
	err := exec.Command(sudoCommand, wazuhControlPath, "stop").Run()
	if err != nil {
		log.Printf("[%s] Failed to pause Wazuh agent: %v\n", time.Now().Format(time.RFC3339), err)
	} else {
		log.Printf("[%s] Wazuh agent paused successfully\n", time.Now().Format(time.RFC3339))
	}
}

// restartAgent restarts the Wazuh agent on macOS
func restartAgent() {
	log.Printf("[%s] Restarting Wazuh agent...\n", time.Now().Format(time.RFC3339))
	err := exec.Command(sudoCommand, wazuhControlPath, "restart").Run()
	if err != nil {
		log.Printf("[%s] Failed to restart Wazuh agent: %v\n", time.Now().Format(time.RFC3339), err)
	} else {
		log.Printf("[%s] Wazuh agent restarted successfully\n", time.Now().Format(time.RFC3339))
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
		log.Printf("Wazuh agent updated successfully\n")
	}
}

func windowsMain() {
	// This function is intentionally left empty for macOS builds.
}