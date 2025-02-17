package main

import (
	"bufio"
	"fmt"
	"log"
	"net"
	"runtime"
	"strings"
	"net/http"
	"io"
	"os/exec"
	"embed"
)

const versionURL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/heads/feat/ota-update/version.txt"
var isUpdateInProgress bool // Flag to track if the update is in progress
var embeddedFiles embed.FS

func main() {

	if runtime.GOOS == "windows" {
		windowsMain()
	} else {
		log.Println("Starting wazuh-agent-status server...")
		listener, err := net.Listen("tcp", ":50505")
		if err != nil {
			log.Fatalf("Failed to start server: %v", err)
		}
		defer listener.Close()
		log.Println("wazuh-agent-status server listening on port 50505")

		for {
			conn, err := listener.Accept()
			if err != nil {
				log.Printf("Failed to accept connection: %v", err)
				continue
			}
			go handleConnection(conn)
		}
	}	
}

func handleConnection(conn net.Conn) {
	defer conn.Close()
	reader := bufio.NewReader(conn)

	for {
		message, err := reader.ReadString('\n')
		if err != nil {
			log.Printf("Connection closed or error: %v", err)
			return
		}

		command := message//[:len(message)-1] // Remove newline character
		command = strings.TrimSpace(command)
		switch command {
		case "status":
			status, connection := checkServiceStatus()
			conn.Write([]byte(fmt.Sprintf("Status: %s, Connection: %s\n", status, connection)))
		case "pause":
			conn.Write([]byte("Pausing the Wazuh Agent...\n"))
			pauseAgent()
			conn.Write([]byte("Paused the Wazuh Agent\n"))
		case "restart":
			conn.Write([]byte("Restarting the Wazuh Agent...\n"))
			restartAgent()
			conn.Write([]byte("Restarted the Wazuh Agent\n"))
		case "update":
			log.Println("Received update command...")
			isUpdateInProgress = true
			updateAgent()
			isUpdateInProgress = false
			log.Println("Update finished")
		case "update-status":
			if isUpdateInProgress {
				conn.Write([]byte("Update: Progressing\n"))
			} else {
				conn.Write([]byte("Update: Disable\n"))
			}
		case "check-version":
			localVersion := getLocalVersion()
			onlineVersion := fetchOnlineVersion()

			if localVersion == "" || onlineVersion == "" {
				conn.Write([]byte("VersionCheck: Unknown\n"))
			} else if localVersion != onlineVersion {
				conn.Write([]byte(fmt.Sprintf("VersionCheck: Outdated (v%s -> v%s)\n", localVersion, onlineVersion)))
			} else {
				conn.Write([]byte(fmt.Sprintf("VersionCheck: Up to date -> v%s\n", localVersion)))
			}
		default:
			conn.Write([]byte(fmt.Sprintf("Unknown command: %s \n", command)))
		}
	}
}

// Run a command as root using sudo
func runAsRoot(command string, args ...string) (string, error) {
	cmd := exec.Command("sudo", append([]string{command}, args...)...)
	output, err := cmd.CombinedOutput()
	return string(output), err
}

// Read local version from embedded file, running as root
func getLocalVersion() string {
	// Use sudo to run the read command with root permissions
	output, err := runAsRoot("cat", getVersionFilePath())
	if err != nil {
		log.Printf("Failed to read local version: %v", err)
		return "Unknown"
	}

	log.Printf("Local version: %v", output)
	return output
}

// Fetch the latest version from the server
func fetchOnlineVersion() string {
	resp, err := http.Get(versionURL)
	if err != nil {
		log.Printf("Failed to fetch online version: %v", err)
		return "Unknown"
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		log.Printf("Failed to read response: %v", err)
		return "Unknown"
	}
	log.Printf("Online version: %v", string(body))
	return strings.TrimSpace(string(body))
}

// getVersionPath returns version file path based on the OS
func getVersionFilePath() string {
    switch os := runtime.GOOS; os {
    case "linux":
        return "/var/ossec/etc/version.txt"
	case "darwin":
		return "/Library/Ossec/etc/version.txt"
	case "windows":
		return "C:\\Program Files (x86)\\ossec-agent\\version.txt"
    default:
        return "/var/ossec/etc/version.txt"
    }
}