package main

import (
    "fmt"
    "io/ioutil"
    "log"
    "os/exec"
    "runtime"
    "strings"
    "time"

    "github.com/getlantern/systray"
)

var statusItem, connectionItem *systray.MenuItem

func main() {
    go func() {
        systray.Run(onReady, onExit)
    }()
    // This keeps the application running in the background.
    select {}
}

func onReady() {
    systray.SetIcon(getIcon("/home/armand-meppa/Pictures/wazuh-logo-min.png"))
    systray.SetTitle("Wazuh Agent")
    systray.SetTooltip("Wazuh Agent Status")

    statusItem = systray.AddMenuItem("Status: Checking...", "Wazuh Agent Status")
    connectionItem = systray.AddMenuItem("Connection: Checking...", "Wazuh Agent Connection")

    quitItem := systray.AddMenuItem("Quit", "Quit the application")

    go func() {
        for {
            updateStatus()
            time.Sleep(5 * time.Second)
        }
    }()

    go func() {
        <-quitItem.ClickedCh
        systray.Quit()
    }()
}

func onExit() {
    // Perform cleanup if necessary
}

func updateStatus() {
    status, connection := checkServiceStatus()
    statusItem.SetTitle(fmt.Sprintf("Status: %s", status))
    connectionItem.SetTitle(fmt.Sprintf("Connection: %s", connection))
}

func checkServiceStatus() (string, string) {
    var statusCmd, connectionCmd *exec.Cmd
    switch runtime.GOOS {
    case "linux":
        statusCmd = exec.Command("sudo", "systemctl", "status", "wazuh-agent.service")
        connectionCmd = exec.Command("sudo", "grep", "^status", "/var/ossec/var/run/wazuh-agentd.state")
    case "darwin":
        statusCmd = exec.Command("sudo ", "/Library/Ossec/bin/wazuh-control", "status")
        connectionCmd = exec.Command("sudo", "grep", "^status", "/Library/Ossec/var/run/wazuh-agentd.state")
    case "windows":
        statusCmd = exec.Command("C:\\Program Files (x86)\\ossec\\bin\\wazuh-control", "status")
        connectionCmd = exec.Command("powershell", "-Command", "Select-String -Path 'C:\\Program Files (x86)\\ossec-agent\\wazuh-agent.state' -Pattern '^status'")
    default:
        return "Unsupported OS", "Unsupported OS"
    }

    statusOutput, statusErr := statusCmd.Output()
    connectionOutput, connectionErr := connectionCmd.Output()

    status := "Inactive"
    connection := "Disconnected"

    if statusErr == nil {
        stdout := string(statusOutput)
        if runtime.GOOS == "linux" {
            if strings.Contains(stdout, "Active: active (running)") {
                status = "Active"
            }
        } else {
            for _, line := range strings.Split(stdout, "\n") {
                if strings.Contains(line, "is running...") {
                    status = "Active"
                    break
                }
            }
        }
    }

    if connectionErr == nil {
        stdout := string(connectionOutput)
        if runtime.GOOS == "windows" {
            if strings.Contains(stdout, "status='connected'") {
                connection = "Connected"
            }
        } else {
            if strings.Contains(stdout, "status='connected'") {
                connection = "Connected"
            }
        }
    }

    return status, connection
}

func getIcon(path string) []byte {
    b, err := ioutil.ReadFile(path)
    if err != nil {
        log.Fatal(err)
    }
    return b
}
