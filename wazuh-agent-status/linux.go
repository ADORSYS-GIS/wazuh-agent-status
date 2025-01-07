//go:build linux
// +build linux

package main

import (
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

// syncAgent reconnects the Wazuh agent to the manager on Linux
func syncAgent() {
	log.Printf("[%s] Syncing Wazuh agent with manager...\n", time.Now().Format(time.RFC3339))
	
	log.Printf("[%s] Deleting client.keys file...\n", time.Now().Format(time.RFC3339))
	err := exec.Command("sudo", "rm", "/var/ossec/etc/client.keys").Run()
	if err != nil {
		log.Printf("[%s] Failed to delete the client.keys file: %v\n", time.Now().Format(time.RFC3339), err)
	} else {
		log.Printf("[%s] client.keys file deleted successfully\n", time.Now().Format(time.RFC3339))
	}
	
	restartAgent()
	
	log.Printf("[%s] Wazuh agent synced successfully\n", time.Now().Format(time.RFC3339))
}

func windowsMain() {

}