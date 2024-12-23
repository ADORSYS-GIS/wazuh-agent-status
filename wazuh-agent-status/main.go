package main

import (
	"bufio"
	"fmt"
	"log"
	"net"
	"runtime"
	"strings"
)

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
			conn.Write([]byte("Updating the Wazuh Agent...\n"))
			updateAgent()
			conn.Write([]byte("Updated the Wazuh Agent\n"))
		default:
			conn.Write([]byte(fmt.Sprintf("Unknown command: %s \n", command)))
		}
	}
}
