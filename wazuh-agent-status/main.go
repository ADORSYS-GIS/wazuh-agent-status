package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"time"

	"gopkg.in/natefinch/lumberjack.v2"
)

const (
	versionURL = "https://api.github.com/repos/ADORSYS-GIS/wazuh-agent/releases/latest"
)

// Version is set at build time via ldflags
var Version = "dev"

type GitHubRelease struct {
	TagName string `json:"tag_name"`
	Prerelease bool `json:"prerelease"`
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
	onlineVersion := fetchOnlineVersion()

	currentVersion := fmt.Sprintf("v%s", localVersion)
	if localVersion == "Unknown" || onlineVersion == "Unknown" {
		currentVersion = "Version: Unknown"
	} else if isVersionHigher(onlineVersion, localVersion) {
		currentVersion = fmt.Sprintf("Outdated, v%s", localVersion)
	} else {
		currentVersion = fmt.Sprintf("Up to date, v%s", localVersion)
	}
	manager.SetVersion(currentVersion)
	log.Printf("Version status: %s", currentVersion)
}

// --- SERVER AND CONNECTION HANDLER ---

func getSystemLogFilePath() string {
	var logDir string

	switch runtime.GOOS {
	case "linux", "darwin":
		logDir = "/var/log"
	case "windows":
		logDir = "C:\\ProgramData\\wazuh\\logs"
	default:
		logDir = "./logs"
	}

	if runtime.GOOS == "windows" {
		if err := os.MkdirAll(logDir, 0755); err != nil {
			log.Fatalf("failed to create log directory: %v", err)
		}
	}

	return filepath.Join(logDir, "wazuh-agent-status.log")
}

func init() {
	logFilePath := getSystemLogFilePath()

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
			checkAndSetVersion() // Check version on demand
			versionInfo := manager.GetVersion()
			conn.Write([]byte(fmt.Sprintf("VERSION_CHECK: %s\n", versionInfo)))

		case "subscribe-status":
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
			return

		case "update":
			log.Println("Received update command. Starting update stream...")
			// Update routine runs in a new goroutine and streams progress
			go func() {
				// We dial self to open a new dedicated connection for the update stream.
				updateConn, dialErr := net.DialTimeout("tcp", "localhost:50505", 5*time.Second)
				if dialErr != nil {
					log.Printf("Failed to dial self for update stream: %v", dialErr)
					conn.Write([]byte("ERROR: Update stream failed to start.\n"))
					return
				}
				defer updateConn.Close()

				// Send the dedicated command to initiate the stream
				fmt.Fprintln(updateConn, "initiate-update-stream")

				// Read the stream response and pipe it to the client's current connection
				_, copyErr := io.Copy(conn, updateConn)
				if copyErr != nil && copyErr != io.EOF {
					log.Printf("Error streaming update logs: %v", copyErr)
				}
			}()

		case "initiate-update-stream":
			// This is the dedicated connection for the update process, passed from the self-dial
			updateAgent(conn) // updateAgent will write progress and close this conn
			log.Println("Update finished and stream closed.")

		default:
			conn.Write([]byte(fmt.Sprintf("ERROR: Unknown command: %s\n", command)))
		}
	}
}

// Run a command as root using sudo
func runAsRoot(command string, args ...string) (string, error) {
	cmd := exec.Command("/usr/bin/sudo", append([]string{command}, args...)...)
	output, err := cmd.CombinedOutput()
	return string(output), err
}

// Read local version from embedded file
func getLocalVersion() string {
	if runtime.GOOS == "windows" {
		output, err := os.ReadFile(getVersionFilePath())
		if err != nil {
			log.Printf("Failed to read local version on Windows: %v", err)
			return "Unknown"
		}
		return strings.TrimSpace(string(output))
	} else {
		output, err := runAsRoot("cat", getVersionFilePath())
		if err != nil {
			log.Printf("Failed to read local version on Linux/macOS: %v", err)
			return "Unknown"
		}
		return strings.TrimSpace(output)
	}
}

func isVersionHigher(online, local string) bool {
	onlineParts := strings.Split(strings.TrimPrefix(online, "v"), ".")
	localParts := strings.Split(strings.TrimPrefix(local, "v"), ".")

	for i := 0; i < len(onlineParts) && i < len(localParts); i++ {
		var onlineNum, localNum int
		fmt.Sscanf(onlineParts[i], "%d", &onlineNum)
		fmt.Sscanf(localParts[i], "%d", &localNum)

		if onlineNum > localNum {
			return true
		}
		if onlineNum < localNum {
			return false
		}
	}

	return len(onlineParts) > len(localParts)
}

func fetchOnlineVersion() string {
	resp, err := http.Get(versionURL)
	if err != nil {
		log.Printf("Failed to fetch online version: %v", err)
		return "Unknown"
	}
	defer resp.Body.Close()

	var release GitHubRelease
	err = json.NewDecoder(resp.Body).Decode(&release)
	log.Printf("Fetched release info: TagName=%s, Prerelease=%v", release.TagName, release.Prerelease)
	if err != nil {
		log.Printf("Failed to parse release: %v", err)
		return "Unknown"
	}

	if release.TagName == "" {
		log.Println("No release found")
		return "Unknown"
	}

	return release.TagName
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