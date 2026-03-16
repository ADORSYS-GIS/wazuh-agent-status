package main

import (
	"bufio"
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"runtime"
	"strings"
	"sync"
	"time"

	"gopkg.in/natefinch/lumberjack.v2"
)

const (
	versionURL           = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/heads/main/versions.json"
	backendPort          = "50505"
	backendAddress       = "localhost:" + backendPort
	sudoCommand          = "/usr/bin/sudo"
	grepCommand          = "/usr/bin/grep"
	errorPrefix          = "ERROR:"
	upToDateStatus       = "Up to date"
	sourceFileMarker     = "Source file:"
	powershellExe        = "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"
	executionPolicyFlag  = "-ExecutionPolicy"
	cmdFlag              = "-Command"
	unsupportedOSMessage = "Unsupported OS"
)

// Version is set at build time via ldflags
var Version = "dev"

type VersionInfo struct {
	Framework struct {
		Version           string `json:"version"`
		PrereleaseVersion string `json:"prerelease_version"`
	} `json:"framework"`
	PrereleaseTestGroups []string `json:"prerelease_test_groups"`
}

// Global state and communication channels
var (
	// Notifier broadcasts status changes to all subscribed clients
	notifier = NewEventNotifier()
	// Manager handles the overall state of the Wazuh Agent
	manager = NewStateManager()
)

// --- STATE MANAGEMENT & NOTIFICATION ---

type AgentState struct {
	Status     string
	Connection string
	Version    string
}

type StateManager struct {
	state AgentState
	mu    sync.RWMutex
}

func NewStateManager() *StateManager {
	return &StateManager{
		state: AgentState{
			Status: "Unknown", Connection: "Unknown", Version: "Unknown",
		},
	}
}

func (m *StateManager) SetStatus(status, connection string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	changed := false
	if m.state.Status != status || m.state.Connection != connection {
		m.state.Status = status
		m.state.Connection = connection
		changed = true
	}
	if changed {
		log.Printf("State change: Status=%s, Connection=%s", status, connection)
		// Push update instantly
		notifier.Notify(fmt.Sprintf("STATUS_UPDATE: %s, %s\n", status, connection))
	}
}

func (m *StateManager) GetStatus() (string, string) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	return m.state.Status, m.state.Connection
}

func (m *StateManager) SetVersion(version string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.state.Version != version {
		m.state.Version = version
	}
}

func (m *StateManager) GetVersion() string {
	m.mu.RLock()
	defer m.mu.RUnlock()
	return m.state.Version
}

// EventNotifier handles subscriptions for event pushing
type EventNotifier struct {
	// A map of connection ID (as a unique string) to the channel it listens on
	subscribers map[string]chan string
	mu          sync.Mutex
}

func NewEventNotifier() *EventNotifier {
	return &EventNotifier{
		subscribers: make(map[string]chan string),
	}
}

func (n *EventNotifier) Subscribe(connID string) chan string {
	n.mu.Lock()
	defer n.mu.Unlock()
	ch := make(chan string, 5) // Buffered channel for safety
	n.subscribers[connID] = ch
	return ch
}

func (n *EventNotifier) Unsubscribe(connID string) {
	n.mu.Lock()
	defer n.mu.Unlock()
	if ch, ok := n.subscribers[connID]; ok {
		close(ch)
		delete(n.subscribers, connID)
	}
}

func (n *EventNotifier) Notify(message string) {
	n.mu.Lock()
	defer n.mu.Unlock()
	for _, ch := range n.subscribers {
		// Non-blocking send to prevent one slow client from blocking others
		select {
		case ch <- message:
		default:
			log.Println("Warning: Dropping message for a slow subscriber.")
		}
	}
}

// --- POLLING ROUTINES ---

// monitorAgentStatus polls the OS for the agent's status and updates the StateManager.
func monitorAgentStatus() {
	for {
		status, connection := checkServiceStatus()
		manager.SetStatus(status, connection)
		time.Sleep(5 * time.Second)
	}
}

func checkAndSetVersion() {
	localVersion := getLocalVersion()
	agentGroups, err := getAgentGroups()
	if err != nil {
		log.Println("Failed to extract agent groups")
		return
	}

	versionInfo := fetchVersionInfo()

	// Check if current version is a prerelease (contains "rc")
	isCurrentPrerelease := strings.Contains(localVersion, "rc")

	var versionPrefix string
	if isCurrentPrerelease {
		versionPrefix = fmt.Sprintf("Prerelease: v%s", localVersion)
	} else {
		versionPrefix = fmt.Sprintf("v%s", localVersion)
	}

	currentVersion := versionPrefix
	if localVersion == "Unknown" || versionInfo == nil {
		currentVersion = "Version: Unknown"
	} else {
		isOutdated := versionInfo.Framework.Version != "" && isVersionHigher(versionInfo.Framework.Version, localVersion)
		hasPrerelease := versionInfo.Framework.PrereleaseVersion != "" && shouldShowPrerelease(versionInfo, agentGroups) && isVersionHigher(versionInfo.Framework.PrereleaseVersion, localVersion)

		if isOutdated && hasPrerelease {
			currentVersion = fmt.Sprintf("Outdated with Prerelease available: %s (stable: %s, prerelease: %s)", versionPrefix, versionInfo.Framework.Version, versionInfo.Framework.PrereleaseVersion)
		} else if isOutdated {
			currentVersion = fmt.Sprintf("Outdated, %s", versionPrefix)
		} else if hasPrerelease {
			currentVersion = fmt.Sprintf("Prerelease available: %s (current: %s)", versionInfo.Framework.PrereleaseVersion, versionPrefix)
		} else {
			currentVersion = fmt.Sprintf("%s, %s", upToDateStatus, versionPrefix)
		}
	}
	manager.SetVersion(currentVersion)
	log.Printf("Version status: %s", currentVersion)
}

// --- SERVER AND CONNECTION HANDLER ---

func init() {
	logFilePath, err := getSystemLogFilePath()
	if err != nil {
		log.Fatalf("Failed to get system log file path: %v", err)
	}

	log.SetOutput(&lumberjack.Logger{
		Filename:   logFilePath,
		MaxSize:    10, // MB
		MaxBackups: 3,
		MaxAge:     28, // days
		Compress:   true,
	})

	log.Printf("Logging to: %s", logFilePath)
}

func main() {
	for _, arg := range os.Args[1:] {
		if arg == "--version" || arg == "-v" {
			fmt.Println(Version)
			return
		}
	}
	fmt.Printf("Starting server... (version: %s)\n", Version)

	// Start polling routines
	go monitorAgentStatus()

	if runtime.GOOS == "windows" {
		windowsMain()
	} else {
		log.Println("Starting wazuh-agent-status server...")
		listener, err := net.Listen("tcp", ":"+backendPort)
		if err != nil {
			log.Fatalf("Failed to start server: %v", err)
		}
		defer listener.Close()
		log.Println("wazuh-agent-status server listening on port " + backendPort)

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
	connID := conn.RemoteAddr().String()
	reader := bufio.NewReader(conn)

	log.Printf("Authorized connection from %s", connID)

	// Read loop
	for {
		message, err := reader.ReadString('\n')
		if err != nil {
			if err != io.EOF && !strings.Contains(err.Error(), "timeout") {
				log.Printf("Connection error for %s: %v", connID, err)
			}
			notifier.Unsubscribe(connID)
			return
		}

		command := strings.TrimSpace(message)
		log.Printf("Received command from %s: %s", connID, command)

		switch command {
		case "get-version":
			handleGetVersion(conn)

		case "subscribe-status":
			handleSubscribeStatus(conn, connID)
			return

		case "update":
			log.Println("Received update command. Starting update stream...")
			startUpdateStreamAsync(conn, false)

		case "update-prerelease":
			log.Println("Received prerelease update command. Starting update stream...")
			startUpdateStreamAsync(conn, true)

		case "initiate-update-stream":
			// This is the dedicated connection for the update process
			updateAgent(conn, false)
			log.Println("Update finished and stream closed.")

		case "initiate-prerelease-update-stream":
			// This is the dedicated connection for the prerelease update process
			updateAgent(conn, true)
			log.Println("Prerelease update finished and stream closed.")

		default:
			conn.Write([]byte(fmt.Sprintf("%s Unknown command: %s\n", errorPrefix, command)))
		}
	}
}

// handleGetVersion processes the get-version command
func handleGetVersion(conn net.Conn) {
	checkAndSetVersion() // Check version on demand
	versionInfo := manager.GetVersion()
	conn.Write([]byte(fmt.Sprintf("VERSION_CHECK: %s\n", versionInfo)))
}

// handleSubscribeStatus processes the subscribe-status command
func handleSubscribeStatus(conn net.Conn, connID string) {
	ch := notifier.Subscribe(connID)

	// Send initial state immediately
	status, connection := manager.GetStatus()
	conn.Write([]byte(fmt.Sprintf("STATUS_UPDATE: %s, %s\n", status, connection)))

	// Push loop
	for update := range ch {
		_, err := conn.Write([]byte(update))
		if err != nil {
			log.Printf("Error writing update to %s: %v", connID, err)
			notifier.Unsubscribe(connID)
			return
		}
	}
}

// startUpdateStreamAsync starts an update stream in a goroutine
func startUpdateStreamAsync(conn net.Conn, isPrerelease bool) {
	go func() {
		updateConn, dialErr := net.DialTimeout("tcp", backendAddress, 5*time.Second)
		if dialErr != nil {
			streamType := "update"
			if isPrerelease {
				streamType = "prerelease update"
			}
			log.Printf("Failed to dial self for %s stream: %v", streamType, dialErr)
			conn.Write([]byte(fmt.Sprintf("%s %s stream failed to start.\n", errorPrefix, streamType)))
			return
		}
		defer updateConn.Close()

		// Send the dedicated command to initiate the stream
		command := "initiate-update-stream"
		if isPrerelease {
			command = "initiate-prerelease-update-stream"
		}
		fmt.Fprintln(updateConn, command)

		// Read the stream response and pipe it to the client's current connection
		_, copyErr := io.Copy(conn, updateConn)
		if copyErr != nil && copyErr != io.EOF {
			log.Printf("Error streaming update logs: %v", copyErr)
		}
	}()
}
