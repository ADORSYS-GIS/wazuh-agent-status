//go:build linux
// +build linux

package main

import (
	"log"
	"os/exec"
	"strings"
	"time"
)

func getIconPath() string {
    return "assets/wazuh-logo.png" // Path to the PNG icon for Linux
}

// checkServiceStatus checks the status of Wazuh agent and its connection on Linux
func checkServiceStatus() (string, string) {
	// Command to check agent status
	cmd := exec.Command("sh", "-c", "/var/ossec/bin/wazuh-control status")
	output, err := cmd.Output()
	if err != nil {
		return "Inactive", "Disconnected"
	}

	status := "Inactive"
	if strings.Contains(string(output), "wazuh-agentd is running") {
		status = "Active"
	}

	// Check connection status
	connCmd := exec.Command("sh", "-c", "grep ^status /var/ossec/var/run/wazuh-agentd.state")
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
	err := exec.Command("/var/ossec/bin/wazuh-control", "stop").Run()
	if err != nil {
		log.Printf("[%s] Failed to pause Wazuh agent: %v\n", time.Now().Format(time.RFC3339), err)
	} else {
		log.Printf("[%s] Wazuh agent paused successfully\n", time.Now().Format(time.RFC3339))
	}
}

// restartAgent restarts the Wazuh agent on Linux
func restartAgent() {
	log.Printf("[%s] Restarting Wazuh agent...\n", time.Now().Format(time.RFC3339))
	err := exec.Command("/var/ossec/bin/wazuh-control", "restart").Run()
	if err != nil {
		log.Printf("[%s] Failed to restart Wazuh agent: %v\n", time.Now().Format(time.RFC3339), err)
	} else {
		log.Printf("[%s] Wazuh agent restarted successfully\n", time.Now().Format(time.RFC3339))
	}
}
