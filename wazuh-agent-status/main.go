package main

import (
	"bufio"
	"fmt"
	"log"
	"net"
)

func main() {
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

func handleConnection(conn net.Conn) {
	defer conn.Close()
	reader := bufio.NewReader(conn)

	for {
		message, err := reader.ReadString('\n')
		if err != nil {
			log.Printf("Connection closed or error: %v", err)
			return
		}

		command := message[:len(message)-1] // Remove newline character
		switch command {
		case "status":
			status, connection := checkServiceStatus()
			conn.Write([]byte(fmt.Sprintf("Status: %s, Connection: %s\n", status, connection)))
		default:
			conn.Write([]byte("Unknown command\n"))
		}
	}
}
