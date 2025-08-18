package main

import (
	"bufio"
	"embed"
	"fmt"
	"log"
	"net"
	"os"
	"path/filepath"
	"runtime"
	"runtime/debug"
	"strings"
	"sync/atomic"
	"time"

	"github.com/getlantern/systray"
	"gopkg.in/natefinch/lumberjack.v2"
)

//go:embed assets/*
var embeddedFiles embed.FS

var (
	statusItem, connectionItem, updateItem, versionItem, memItem *systray.MenuItem
	enabledIcon, disabledIcon                                    []byte

	// Track last values to avoid redundant systray updates (which can leak native resources)
	lastStatus     string
	lastConnection string
	lastVersion    string
	lastUpdating   bool

	// update monitor on/off (use atomic for safety)
	isMonitoringUpdate atomic.Bool
)

// Version is set at build time via ldflags
var Version = "dev"

const (
	backendAddr              = "localhost:50505"
	dialTimeout              = 2 * time.Second
	ioDeadline               = 3 * time.Second
	statusInterval           = 10 * time.Second // was 5s; fewer updates = fewer allocs
	versionInterval          = 4 * time.Hour
	memInterval              = 30 * time.Second
	updatePollInterval       = 5 * time.Second
	updateMonitorMaxDuration = 15 * time.Minute // hard cap; avoids runaway goroutine
)

func getUserLogFilePath() string {
	var logDir string

	switch runtime.GOOS {
	case "linux", "darwin":
		logDir = filepath.Join(os.Getenv("HOME"), ".wazuh")
	case "windows":
		logDir = filepath.Join(os.Getenv("APPDATA"), "wazuh", "logs")
	default:
		logDir = "./logs"
	}

	if err := os.MkdirAll(logDir, 0o755); err != nil {
		log.Fatalf("failed to create log directory: %v", err)
	}
	return filepath.Join(logDir, "wazuh-agent-status-client.log")
}

func init() {
	logFilePath := getUserLogFilePath()
	log.SetOutput(&lumberjack.Logger{
		Filename:   logFilePath,
		MaxSize:    10, // MB
		MaxBackups: 3,
		MaxAge:     30, // days
		Compress:   true,
	})

	// Make GC a bit more responsive in long-running tray apps
	debug.SetGCPercent(100)
}

func main() {
	for _, arg := range os.Args[1:] {
		if arg == "--version" || arg == "-v" {
			fmt.Println(Version)
			return
		}
	}

	log.Printf("Starting frontend... (version: %s)", Version)
	systray.Run(onReady, onExit)
}

func onReady() {
	mainIcon, err := getEmbeddedFile(getIconPath())
	if err != nil {
		log.Fatalf("Failed to load main icon: %v", err)
	}
	systray.SetIcon(mainIcon)
	systray.SetTooltip("Wazuh Agent Status")

	enabledIcon, _ = getEmbeddedFile("assets/green-dot.png")
	disabledIcon, _ = getEmbeddedFile("assets/gray-dot.png")

	statusItem = systray.AddMenuItem("Agent: Unknown", "Wazuh Agent Status")
	statusItem.Disable()
	connectionItem = systray.AddMenuItem("Connection: Unknown", "Wazuh Agent Connection")
	connectionItem.Disable()
	systray.AddSeparator()
	updateItem = systray.AddMenuItem("---", "Update the Wazuh Agent")
	updateItem.Disable()
	systray.AddSeparator()
	versionItem = systray.AddMenuItem("v---", "Version of Wazuh setup")
	versionItem.Disable()
	memItem = systray.AddMenuItem("Mem: -- MB", "Process memory (RSS approx)")
	memItem.Disable()

	go handleMenuActions()

	// Single background loop for all periodic work
	go func() {
		statusTicker := time.NewTicker(statusInterval)
		versionTicker := time.NewTicker(versionInterval)
		memTicker := time.NewTicker(memInterval)

		// immediate first run
		updateAgentStatus()
		checkVersion()
		updateMemoryMenu()

		defer statusTicker.Stop()
		defer versionTicker.Stop()
		defer memTicker.Stop()

		for {
			select {
			case <-statusTicker.C:
				updateAgentStatus()
			case <-versionTicker.C:
				checkVersion()
			case <-memTicker.C:
				updateMemoryMenu()
			}
		}
	}()
}

func updateAgentStatus() {
	status, connection := fetchStatus()

	// Only update UI if changed (prevents repeated native allocations)
	if status != lastStatus {
		lastStatus = status
		if status == "Active" {
			statusItem.SetTitle("Agent: Active")
			statusItem.SetIcon(enabledIcon)
		} else if status == "Inactive" {
			statusItem.SetTitle("Agent: Inactive")
			statusItem.SetIcon(disabledIcon)
		} else {
			statusItem.SetTitle("Agent: Unknown")
			statusItem.SetIcon(disabledIcon)
		}
	}

	if connection != lastConnection {
		lastConnection = connection
		if connection == "Connected" {
			connectionItem.SetTitle("Connection: Connected")
			connectionItem.SetIcon(enabledIcon)
		} else if connection == "Disconnected" {
			connectionItem.SetTitle("Connection: Disconnected")
			connectionItem.SetIcon(disabledIcon)
		} else {
			connectionItem.SetTitle("Connection: Unknown")
			connectionItem.SetIcon(disabledIcon)
		}
	}

	// If version item was never set (first run) or unknown, try to fill it
	if lastVersion == "" || strings.Contains(lastVersion, "Unknown") {
		checkVersionAfterUpdate()
	}
}

func checkVersion() {
	versionStatus, version := fetchVersionStatus()
	lastVersion = version

	switch {
	case strings.HasPrefix(versionStatus, "Up to date"):
		versionItem.SetTitle(version)
		updateItem.SetTitle("Up to date")
		updateItem.Disable()
		setUpdating(false)
	case strings.HasPrefix(versionStatus, "Outdated"):
		versionItem.SetTitle(version)
		updateItem.SetTitle("Update")
		updateItem.Enable()
		log.Println("Version is outdated, starting update monitor...")
		startUpdateMonitor()
	default:
		versionItem.SetTitle("Version: Unknown")
		updateItem.Disable()
	}
}

func checkVersionAfterUpdate() {
	versionStatus, version := fetchVersionStatus()
	lastVersion = version

	switch {
	case strings.HasPrefix(versionStatus, "Up to date"):
		versionItem.SetTitle(version)
		updateItem.SetTitle("Up to date")
		updateItem.Disable()
		setUpdating(false)
	case strings.HasPrefix(versionStatus, "Outdated"):
		versionItem.SetTitle(version)
		updateItem.SetTitle("Update")
		updateItem.Enable()
	default:
		versionItem.SetTitle("Version: Unknown")
		updateItem.Disable()
	}
}

func startUpdateMonitor() {
	// ensure single monitor
	if !isMonitoringUpdate.CompareAndSwap(false, true) {
		return
	}

	// Trigger update once
	sendCommand("update")
	setUpdating(true)

	go func() {
		defer isMonitoringUpdate.Store(false)

		t0 := time.Now()
		ticker := time.NewTicker(updatePollInterval)
		defer ticker.Stop()

		for {
			select {
			case <-ticker.C:
				updateStatus := fetchUpdateStatus()

				if updateStatus == "Disable" || updateStatus == "Disabled" || updateStatus == "Idle" {
					setUpdating(false)
					checkVersionAfterUpdate()
					return
				}

				// keep the UI informative while updating
				updateItem.SetTitle("Updating...")
				updateItem.Disable()

				// hard stop to avoid runaway goroutine
				if time.Since(t0) > updateMonitorMaxDuration {
					log.Printf("Update monitor timed out after %s, stopping.", updateMonitorMaxDuration)
					setUpdating(false)
					return
				}
			}
		}
	}()
}

func setUpdating(on bool) {
	if lastUpdating == on {
		return
	}
	lastUpdating = on
	if on {
		updateItem.SetTitle("Updating...")
		updateItem.Disable()
	} else {
		// title will be set by version check afterward
	}
}

func handleMenuActions() {
	for range updateItem.ClickedCh {
		startUpdateMonitor()
	}
}

/*** ---- Backend communication helpers with timeouts ---- ***/

func fetchWithTimeout(cmd string, deadline time.Duration) (string, error) {
	conn, err := net.DialTimeout("tcp", backendAddr, dialTimeout)
	if err != nil {
		return "", fmt.Errorf("connect: %w", err)
	}
	defer conn.Close()

	_ = conn.SetDeadline(time.Now().Add(deadline))

	if _, err := fmt.Fprintln(conn, cmd); err != nil {
		return "", fmt.Errorf("write: %w", err)
	}

	reader := bufio.NewReader(conn)
	resp, err := reader.ReadString('\n')
	if err != nil {
		return "", fmt.Errorf("read: %w", err)
	}
	return strings.TrimSpace(resp), nil
}

func fetchStatus() (string, string) {
	resp, err := fetchWithTimeout("status", ioDeadline)
	if err != nil {
		log.Printf("Failed to fetch status: %v", err)
		return "Unknown", "Unknown"
	}

	parts := strings.Split(resp, ", ")
	if len(parts) < 2 {
		return "Unknown", "Unknown"
	}

	statusKV := strings.SplitN(parts[0], ": ", 2)
	connKV := strings.SplitN(parts[1], ": ", 2)
	if len(statusKV) < 2 || len(connKV) < 2 {
		return "Unknown", "Unknown"
	}
	return statusKV[1], connKV[1]
}

func fetchVersionStatus() (string, string) {
	resp, err := fetchWithTimeout("check-version", ioDeadline)
	if err != nil {
		log.Printf("Failed to fetch version status: %v", err)
		return "Unknown", "Unknown"
	}

	// expected: "check-version: Up to date, vX.Y.Z" or "check-version: Outdated, vA.B.C"
	parts := strings.SplitN(resp, ": ", 2)
	if len(parts) < 2 {
		return "Unknown", "Unknown"
	}
	rest := strings.Split(parts[1], ", ")
	if len(rest) < 2 {
		return "Unknown", "Unknown"
	}
	return rest[0], rest[1]
}

func fetchUpdateStatus() string {
	resp, err := fetchWithTimeout("update-status", ioDeadline)
	if err != nil {
		log.Printf("Failed to fetch update status: %v", err)
		return "Unknown"
	}
	// expected: "update-status: <State>"
	parts := strings.SplitN(resp, ": ", 2)
	if len(parts) < 2 {
		return "Unknown"
	}
	return parts[1]
}

func sendCommand(command string) {
	_, err := fetchWithTimeout(command, ioDeadline)
	if err != nil {
		log.Printf("Failed to send command %q: %v", command, err)
	}
}

/*** ---- UI helpers ---- ***/

func getEmbeddedFile(path string) ([]byte, error) {
	return embeddedFiles.ReadFile(path)
}

func getIconPath() string {
	if runtime.GOOS == "windows" {
		return "assets/wazuh-logo.ico"
	}
	return "assets/wazuh-logo.png"
}

func updateMemoryMenu() {
	var m runtime.MemStats
	runtime.ReadMemStats(&m)
	// HeapAlloc is a good proxy for live heap; convert to MB
	mb := float64(m.HeapAlloc) / (1024 * 1024)
	memItem.SetTitle(fmt.Sprintf("Mem: %.1f MB", mb))
	// keep tooltip handy with version + mem
	systray.SetTooltip(fmt.Sprintf("Wazuh Agent Status • v%s • Mem %.1f MB", Version, mb))
}

func onExit() {
	log.Println("Frontend application stopped")
}