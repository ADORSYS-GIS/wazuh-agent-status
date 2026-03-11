package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"time"
)

func getSystemLogFilePath() (string, error) {
	var logDir string

	switch runtime.GOOS {
	case "linux", "darwin":
		logDir = "/var/log"
	case "windows":
		logDir = "C:\\ProgramData\\wazuh\\logs"
	default:
		return "", fmt.Errorf("unsupported OS: %s", runtime.GOOS)
	}

	if runtime.GOOS == "windows" {
		if err := os.MkdirAll(logDir, 0755); err != nil {
			return "", fmt.Errorf("failed to create log directory: %v", err)
		}
	}

	return filepath.Join(logDir, "wazuh-agent-status.log"), nil
}

// Run a command as root using sudo
func runAsRoot(command string, args ...string) (string, error) {
	cmd := exec.Command(sudoCommand, append([]string{command}, args...)...)
	output, err := cmd.CombinedOutput()
	return string(output), err
}

// Read local version from embedded file
func getLocalVersion() string {
	if runtime.GOOS == "windows" {
		versionPath, err := getVersionFilePath()
		if err != nil {
			log.Printf("Failed to get version file path on Windows: %v", err)
			return "Unknown"
		}
		output, err := os.ReadFile(versionPath)
		if err != nil {
			log.Printf("Failed to read local version on Windows: %v", err)
			return "Unknown"
		}
		return strings.TrimSpace(string(output))
	} else {
		versionPath, err := getVersionFilePath()
		if err != nil {
			log.Printf("Failed to get version file path on Linux/macOS: %v", err)
			return "Unknown"
		}
		output, err := runAsRoot("cat", versionPath)
		if err != nil {
			log.Printf("Failed to read local version on Linux/macOS: %v", err)
			return "Unknown"
		}
		return strings.TrimSpace(output)
	}
}

func isVersionHigher(online, local string) bool {
	// Strip "v" prefix
	online = strings.TrimPrefix(online, "v")
	local = strings.TrimPrefix(local, "v")

	// Split by "-" to separate version from prerelease (e.g., "1.9.0-rc.1" -> ["1.9.0", "rc.1"])
	onlineBase := strings.Split(online, "-")[0]
	localBase := strings.Split(local, "-")[0]

	onlineParts := strings.Split(onlineBase, ".")
	localParts := strings.Split(localBase, ".")

	// Compare base version numbers
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

	// If base versions are equal, check length
	if len(onlineParts) != len(localParts) {
		return len(onlineParts) > len(localParts)
	}

	// If base versions are identical, stable release (no "-") is higher than prerelease (has "-")
	// e.g., "1.9.0" > "1.9.0-rc.1"
	onlineIsPrerelease := strings.Contains(online, "-")
	localIsPrerelease := strings.Contains(local, "-")

	if onlineIsPrerelease && !localIsPrerelease {
		return false // online is prerelease, local is stable -> online is NOT higher
	}
	if !onlineIsPrerelease && localIsPrerelease {
		return true // online is stable, local is prerelease -> online IS higher
	}

	// Both are prerelease or both are stable with same base version
	return false
}

func fetchVersionInfo() *VersionInfo {
	// Create HTTP client with timeout to prevent hanging
	client := &http.Client{
		Timeout: 10 * time.Second,
	}

	resp, err := client.Get(versionURL)
	if err != nil {
		log.Printf("Failed to fetch version info: %v", err)
		return nil
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		log.Printf("Failed to fetch version info: HTTP %d", resp.StatusCode)
		return nil
	}

	var versionInfo VersionInfo
	err = json.NewDecoder(resp.Body).Decode(&versionInfo)
	if err != nil {
		log.Printf("Failed to parse version info: %v", err)
		return nil
	}
	log.Printf("Fetched version info: Framework.Version=%s, Framework.PrereleaseVersion=%s",
		versionInfo.Framework.Version, versionInfo.Framework.PrereleaseVersion)

	return &versionInfo
}

// getAgentGroups extracts groups from merged.mg file
func getAgentGroups() ([]string, error) {
	mergedMgPath, err := getMergedMgPath()
	if err != nil {
		log.Printf("Failed to get merged.mg path: %v", err)
		return []string{}, err
	}

	f, err := os.Open(mergedMgPath)
	if err != nil {
		return nil, fmt.Errorf("open merged.mg: %w", err)
	}
	defer f.Close()

	var groups []string
	scanner := bufio.NewScanner(f)

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" {
			continue
		}

		// Case 1: single group — "#SIEM"
		if strings.HasPrefix(line, "#") && !strings.Contains(line, sourceFileMarker) {
			if candidate := strings.TrimSpace(strings.TrimPrefix(line, "#")); candidate != "" {
				groups = append(groups, candidate)
			}
			continue
		}

		// Case 2: multiple groups — "<!-- Source file: <group>/agent.conf -->"
		if strings.Contains(line, sourceFileMarker) {
			parts := strings.SplitN(line, sourceFileMarker, 2)
			if len(parts) == 2 {
				if before, found := strings.CutSuffix(strings.TrimSpace(parts[1]), "/agent.conf -->"); found {
					if group := strings.TrimSpace(before); group != "" {
						groups = append(groups, group)
					}
				}
			}
		}
	}
	log.Printf("Extracted groups: %s", groups)

	return groups, scanner.Err()
}

// shouldShowPrerelease checks if prerelease should be shown based on agent groups
func shouldShowPrerelease(versionInfo *VersionInfo, agentGroups []string) bool {
	if len(versionInfo.PrereleaseTestGroups) == 0 || len(agentGroups) == 0 {
		return false
	}

	// Check if any agent group matches the test groups
	for _, agentGroup := range agentGroups {
		for _, testGroup := range versionInfo.PrereleaseTestGroups {
			if strings.EqualFold(agentGroup, testGroup) {
				return true
			}
		}
	}

	return false
}

func getBasePath() (string, error) {
	switch runtime.GOOS {
	case "linux":
		return "/var/ossec", nil
	case "darwin":
		return "/Library/Ossec", nil
	case "windows":
		return "C:\\Program Files (x86)\\ossec-agent", nil
	default:
		return "", fmt.Errorf("unsupported OS: %s", runtime.GOOS)
	}
}

// getMergedMgPath returns merged.mg path based on the OS
func getMergedMgPath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	switch os := runtime.GOOS; os {
	case "windows":
		return filepath.Join(basePath, "shared", "merged.mg"), nil
	default:
		return filepath.Join(basePath, "etc", "shared", "merged.mg"), nil
	}
}

// getVersionFilePath returns version.txt path based on the OS
func getVersionFilePath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	switch runtime.GOOS {
	case "windows":
		return filepath.Join(basePath, "version.txt"), nil
	default:
		return filepath.Join(basePath, "etc", "version.txt"), nil
	}
}

// getAdorsysUpdatePath returns adorsys-update script path based on the OS
func getAdorsysUpdatePath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	switch runtime.GOOS {
	case "windows":
		return filepath.Join(basePath, "active-response", "bin", "adorsys-update.bat"), nil
	default:
		return filepath.Join(basePath, "active-response", "bin", "adorsys-update.sh"), nil
	}
}

// getVersionFilePath returns state file path based on the OS
func getWazuhStatePath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	switch runtime.GOOS {
	case "windows":
		return filepath.Join(basePath, "wazuh-agent.state"), nil
	default:
		return filepath.Join(basePath, "var", "run", "wazuh-agentd.state"), nil
	}
}

// createLogFile creates a secure log file for update operations
func createLogFile() (*os.File, error) {
	logFileHandle, err := os.CreateTemp("", "wazuh-update-*.log")
	if err != nil {
		log.Printf("ERROR: Failed to create log file: %v", err)
		return nil, err
	}
	logFile := logFileHandle.Name()
	log.Printf("Logging to: %s", logFile)
	return logFileHandle, nil
}

// downloadFile downloads a file from URL to the specified path
func downloadAndSaveFile(url string, filePath string, mod os.FileMode) error {
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("HTTP %d: %s", resp.StatusCode, resp.Status)
	}

	out, err := os.Create(filePath)
	if err != nil {
		return err
	}
	defer out.Close()

	// Only apply chmod on non-Windows systems
	if runtime.GOOS != "windows" {
		if err := os.Chmod(filePath, mod); err != nil {
			return err
		}
	}

	_, err = io.Copy(out, resp.Body)
	return err
}
