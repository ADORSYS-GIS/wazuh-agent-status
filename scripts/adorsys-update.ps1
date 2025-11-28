#requires -version 5.1
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

# ---- Elevate ----
$IsAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $IsAdmin) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName  = (Get-Process -Id $PID).Path
    $psi.Arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$($MyInvocation.MyCommand.Path)`""
    $psi.Verb      = "runas"
    try {
        [System.Diagnostics.Process]::Start($psi) | Out-Null
        exit
    } catch {
        [System.Windows.Forms.MessageBox]::Show("Administrator approval is required. Exiting.","Wazuh Agent Installer",[System.Windows.Forms.MessageBoxButtons]::OK,[System.Windows.Forms.MessageBoxIcon]::Warning) | Out-Null
        exit 1
    }
}

Set-StrictMode -Version Latest

# ---- Configuration Variables ----
$LOG_LEVEL = if ($env:LOG_LEVEL) { $env:LOG_LEVEL } else { "INFO" }
$WAZUH_MANAGER = if ($env:WAZUH_MANAGER) { $env:WAZUH_MANAGER } else { "wazuh.example.com" }
$WAZUH_AGENT_VERSION = if ($env:WAZUH_AGENT_VERSION) { $env:WAZUH_AGENT_VERSION } else { "4.12.0-1" }
$OSSEC_PATH = "C:\Program Files (x86)\ossec-agent\"
$OSSEC_CONF_PATH = Join-Path -Path $OSSEC_PATH -ChildPath "ossec.conf"
$RepoUrl = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/main"
$VERSION_FILE_URL = "$RepoUrl/version.txt"
$VERSION_FILE_PATH = Join-Path -Path $OSSEC_PATH -ChildPath "version.txt"
$WAZUH_YARA_VERSION = if ($env:WAZUH_YARA_VERSION) { $env:WAZUH_YARA_VERSION } else { "0.3.11" }
$WAZUH_SNORT_VERSION = if ($env:WAZUH_SNORT_VERSION) { $env:WAZUH_SNORT_VERSION } else { "0.2.4" }
$WAZUH_AGENT_STATUS_VERSION = if ($env:WAZUH_AGENT_STATUS_VERSION) { $env:WAZUH_AGENT_STATUS_VERSION } else { "0.3.3" }
$WAZUH_SURICATA_VERSION = if ($env:WAZUH_SURICATA_VERSION) { $env:WAZUH_SURICATA_VERSION } else { "0.1.4" }

# ---- Globals ----
$AppName = "Wazuh Agent"
$ActiveResponsesLogDir = Join-Path $OSSEC_PATH "active-response"
$LogPath = Join-Path $ActiveResponsesLogDir "active-responses.log"
$global:InstallerFiles = @()
$global:CurrentStep = 1
$global:InstallationComplete = $false
$global:DetectedManagerAddress = $null

# ---- Logging ----
function Append-Log {
    param(
        [string]$Message,
        [string]$Level = "INFO"
    )
    $ts = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
    $line = "[$ts] [$Level] $Message"
    $LogBox.AppendText($line + [Environment]::NewLine)
    $LogBox.ScrollToCaret()

    # Write to active-responses.log (create directory if needed)
    try {
        # Create active-response directory if it doesn't exist
        if (-not (Test-Path $ActiveResponsesLogDir)) {
            New-Item -ItemType Directory -Force -Path $ActiveResponsesLogDir -ErrorAction Stop | Out-Null
        }

        # Use FileStream with shared access to write to the log file
        $fileStream = $null
        $streamWriter = $null
        try {
            # Open file with shared read/write access so Wazuh agent can still access it
            $fileStream = [System.IO.FileStream]::new(
                $LogPath,
                [System.IO.FileMode]::Append,
                [System.IO.FileAccess]::Write,
                [System.IO.FileShare]::ReadWrite
            )
            $streamWriter = [System.IO.StreamWriter]::new($fileStream, [System.Text.Encoding]::UTF8)
            $streamWriter.WriteLine($line)
            $streamWriter.Flush()
        } finally {
            if ($streamWriter) { $streamWriter.Dispose() }
            if ($fileStream) { $fileStream.Dispose() }
        }
    } catch {
        # Silently ignore errors if still locked
    }

    [System.Windows.Forms.Application]::DoEvents()
}

function InfoMessage {
    param ([string]$Message)
    Append-Log $Message "INFO"
}

function WarningMessage {
    param ([string]$Message)
    Append-Log $Message "WARNING"
}

function SuccessMessage {
    param ([string]$Message)
    Append-Log $Message "SUCCESS"
}

function ErrorMessage {
    param ([string]$Message)
    Append-Log $Message "ERROR"
}

function SectionSeparator {
    param ([string]$SectionName)
    Append-Log "=================================================="
    Append-Log "  $SectionName"
    Append-Log "=================================================="
}

function Invoke-Step {
    param(
        [Parameter(Mandatory=$true)][string]$Name,
        [Parameter(Mandatory=$true)][scriptblock]$Action,
        [int]$Weight = 10
    )
    InfoMessage "[START] $Name"
    $StatusLabel.Text = "Step: $Name"
    try {
        & $Action
        SuccessMessage "[OK] $Name"
    } catch {
        ErrorMessage "[FAIL] $Name : $($_.Exception.Message)"
        throw
    } finally {
        $ProgressBar.Value = [Math]::Min($ProgressBar.Value + $Weight, $ProgressBar.Maximum)
    }
}

# ---- IDS Detection Functions ----
function Test-SnortInstalled {
    # Check common Snort installation paths
    $snortPaths = @(
        "C:\Snort\bin\snort.exe",
        "C:\Program Files\Snort\bin\snort.exe",
        "C:\Program Files (x86)\Snort\bin\snort.exe"
    )

    foreach ($path in $snortPaths) {
        if (Test-Path $path) {
            InfoMessage "Detected existing Snort installation at: $path"
            return $true
        }
    }

    # Check if Snort is in PATH
    try {
        $snortCmd = Get-Command snort -ErrorAction SilentlyContinue
        if ($snortCmd) {
            InfoMessage "Detected Snort in system PATH: $($snortCmd.Source)"
            return $true
        }
    } catch {}

    return $false
}

function Test-SuricataInstalled {
    # Check common Suricata installation paths
    $suricataPaths = @(
        "C:\Program Files\Suricata\suricata.exe",
        "C:\Program Files (x86)\Suricata\suricata.exe",
        "C:\Suricata\suricata.exe"
    )

    foreach ($path in $suricataPaths) {
        if (Test-Path $path) {
            InfoMessage "Detected existing Suricata installation at: $path"
            return $true
        }
    }

    # Check if Suricata is in PATH
    try {
        $suricataCmd = Get-Command suricata -ErrorAction SilentlyContinue
        if ($suricataCmd) {
            InfoMessage "Detected Suricata in system PATH: $($suricataCmd.Source)"
            return $true
        }
    } catch {}

    return $false
}

function Get-CurrentWazuhManager {
    if (Test-Path $OSSEC_CONF_PATH) {
        try {
            InfoMessage "Reading ossec.conf from: $OSSEC_CONF_PATH"
            [xml]$ossecConfig = Get-Content -Path $OSSEC_CONF_PATH -ErrorAction Stop

            # Navigate to the manager address in the XML structure
            $managerAddress = $ossecConfig.ossec_config.client.server.address

            if ($managerAddress) {
                InfoMessage "Detected Wazuh Manager Address: $managerAddress"
                return $managerAddress
            } else {
                WarningMessage "Manager address not found in ossec.conf"
                return $null
            }
        } catch {
            WarningMessage "Failed to parse ossec.conf: $($_.Exception.Message)"
            return $null
        }
    } else {
        WarningMessage "ossec.conf not found at: $OSSEC_CONF_PATH"
        return $null
    }
}

function Test-YaraInstalled {
    $activeResponseBinDir = "C:\Program Files (x86)\ossec-agent\active-response\bin"
    $yaraExePath = Join-Path -Path $activeResponseBinDir -ChildPath "yara\yara64.exe"
    $yaraBatPath = Join-Path -Path $activeResponseBinDir -ChildPath "yara.bat"

    foreach ($path in @($yaraExePath, $yaraBatPath)) {
        if (Test-Path $path) {
            InfoMessage "Detected existing YARA installation at: $path"
            return $true
        }
    }

    try {
        $yaraCmd = Get-Command yara64 -ErrorAction SilentlyContinue
        if ($yaraCmd) {
            InfoMessage "Detected YARA in system PATH: $($yaraCmd.Source)"
            return $true
        }
    } catch {}

    return $false
}

function Set-DefaultIDS {
    InfoMessage "Detecting installed IDS systems..."

    $snortInstalled = Test-SnortInstalled
    $suricataInstalled = Test-SuricataInstalled

    if ($snortInstalled -and $suricataInstalled) {
        InfoMessage "Both Snort and Suricata detected. Defaulting to Suricata."
        $SuricataRadio.Checked = $true
        $SnortRadio.Checked = $false
    } elseif ($snortInstalled) {
        InfoMessage "Snort detected. Setting Snort as default."
        $SnortRadio.Checked = $true
        $SuricataRadio.Checked = $false
    } elseif ($suricataInstalled) {
        InfoMessage "Suricata detected. Setting Suricata as default."
        $SuricataRadio.Checked = $true
        $SnortRadio.Checked = $false
    } else {
        InfoMessage "No existing IDS detected. Defaulting to Suricata."
        $SuricataRadio.Checked = $true
        $SnortRadio.Checked = $false
    }
}

# ---- Cleanup ----
function Cleanup-Installers {
    foreach ($file in $global:InstallerFiles) {
        if (Test-Path $file) {
            Remove-Item $file -Force
            InfoMessage "Removed installer file: $file"
        }
    }
}

# ---- Installation Functions ----
function Install-Dependencies {
    $InstallerURL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/main/scripts/deps.ps1"
    $InstallerPath = "$env:TEMP\deps.ps1"
    $global:InstallerFiles += $InstallerPath

    InfoMessage "Downloading dependency script..."
    Invoke-WebRequest -Uri $InstallerURL -OutFile $InstallerPath -ErrorAction Stop
    InfoMessage "Executing dependency script..."

    # Capture all output streams
    $output = & powershell.exe -ExecutionPolicy Bypass -File $InstallerPath 2>&1
    foreach ($line in $output) {
        if ($line -is [System.Management.Automation.ErrorRecord]) {
            ErrorMessage $line.ToString()
        } else {
            InfoMessage $line.ToString()
        }
    }

    if ($LASTEXITCODE -and $LASTEXITCODE -ne 0) {
        throw "Dependency script failed with exit code $LASTEXITCODE"
    }
}

function Install-WazuhAgent {
    param(
        [string]$ManagerAddress
    )

    $InstallerURL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/main/scripts/install.ps1"
    $InstallerPath = "$env:TEMP\install.ps1"
    $global:InstallerFiles += $InstallerPath

    InfoMessage "Downloading Wazuh agent script..."
    Invoke-WebRequest -Uri $InstallerURL -OutFile $InstallerPath -ErrorAction Stop

    # Determine which manager address to use
    if ($ManagerAddress) {
        InfoMessage "Installing Wazuh agent with manager: $ManagerAddress"
    } else {
        InfoMessage "Installing Wazuh agent with default manager: $WAZUH_MANAGER"
        $ManagerAddress = $WAZUH_MANAGER
    }

    # Build command with environment variable for manager address
    $envVarCommand = "`$env:WAZUH_MANAGER='$ManagerAddress'; & `"$InstallerPath`""

    $process = Start-Process -FilePath "powershell.exe" -ArgumentList "-ExecutionPolicy Bypass -Command `"$envVarCommand`"" -NoNewWindow -PassThru -RedirectStandardOutput "$env:TEMP\wazuh_output.log" -RedirectStandardError "$env:TEMP\wazuh_error.log" -Wait

    if (Test-Path "$env:TEMP\wazuh_output.log") {
        Get-Content "$env:TEMP\wazuh_output.log" | ForEach-Object { InfoMessage $_ }
        Remove-Item "$env:TEMP\wazuh_output.log" -Force
    }
    if (Test-Path "$env:TEMP\wazuh_error.log") {
        Get-Content "$env:TEMP\wazuh_error.log" | ForEach-Object { ErrorMessage $_ }
        Remove-Item "$env:TEMP\wazuh_error.log" -Force
    }

    if ($process.ExitCode -ne 0) {
        throw "Wazuh agent installation failed with exit code $($process.ExitCode)"
    }
}

function Install-Yara {
    $YaraUrl = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-yara/refs/tags/v$WAZUH_YARA_VERSION/scripts/install.ps1"
    $YaraScript = "$env:TEMP\install_yara.ps1"
    $global:InstallerFiles += $YaraScript

    InfoMessage "Downloading YARA script..."
    Invoke-WebRequest -Uri $YaraUrl -OutFile $YaraScript -ErrorAction Stop
    InfoMessage "Installing YARA..."

    $process = Start-Process -FilePath "powershell.exe" -ArgumentList "-ExecutionPolicy Bypass -File `"$YaraScript`"" -NoNewWindow -PassThru -RedirectStandardOutput "$env:TEMP\yara_output.log" -RedirectStandardError "$env:TEMP\yara_error.log" -Wait

    if (Test-Path "$env:TEMP\yara_output.log") {
        Get-Content "$env:TEMP\yara_output.log" | ForEach-Object { InfoMessage $_ }
        Remove-Item "$env:TEMP\yara_output.log" -Force
    }
    if (Test-Path "$env:TEMP\yara_error.log") {
        Get-Content "$env:TEMP\yara_error.log" | ForEach-Object { ErrorMessage $_ }
        Remove-Item "$env:TEMP\yara_error.log" -Force
    }

    if ($process.ExitCode -ne 0) {
        throw "YARA installation failed with exit code $($process.ExitCode)"
    }
}

function Uninstall-Yara {
    $YaraUrl = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-yara/refs/tags/v$WAZUH_YARA_VERSION/scripts/uninstall.ps1"
    $UninstallYaraScript = "$env:TEMP\uninstall_yara.ps1"
    $global:InstallerFiles += $UninstallYaraScript

    # Check if YARA is installed before attempting uninstall
    if (Test-YaraInstalled) {
        InfoMessage "Removing existing YARA installation..."
        Invoke-WebRequest -Uri $YaraUrl -OutFile $UninstallYaraScript -ErrorAction Stop

        $process = Start-Process -FilePath "powershell.exe" -ArgumentList "-ExecutionPolicy Bypass -File `"$UninstallYaraScript`"" -NoNewWindow -PassThru -RedirectStandardOutput "$env:TEMP\uninstall_yara_output.log" -RedirectStandardError "$env:TEMP\uninstall_yara_error.log" -Wait

        if (Test-Path "$env:TEMP\uninstall_yara_output.log") {
            Get-Content "$env:TEMP\uninstall_yara_output.log" | ForEach-Object { InfoMessage $_ }
            Remove-Item "$env:TEMP\uninstall_yara_output.log" -Force
        }
        if (Test-Path "$env:TEMP\uninstall_yara_error.log") {
            Get-Content "$env:TEMP\uninstall_yara_error.log" | ForEach-Object { ErrorMessage $_ }
            Remove-Item "$env:TEMP\uninstall_yara_error.log" -Force
        }

        if ($process.ExitCode -ne 0) {
            WarningMessage "YARA uninstall completed with exit code $($process.ExitCode)"
        }
    } else {
        InfoMessage "YARA is not installed. Skipping uninstall."
    }
}

function Install-Snort {
    $SnortUrl = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-snort/refs/tags/v$WAZUH_SNORT_VERSION/scripts/windows/snort.ps1"
    $SnortScript = "$env:TEMP\snort.ps1"
    $global:InstallerFiles += $SnortScript

    InfoMessage "Downloading Snort script..."
    Invoke-WebRequest -Uri $SnortUrl -OutFile $SnortScript -ErrorAction Stop
    InfoMessage "Installing Snort..."

    $process = Start-Process -FilePath "powershell.exe" -ArgumentList "-ExecutionPolicy Bypass -File `"$SnortScript`"" -NoNewWindow -PassThru -RedirectStandardOutput "$env:TEMP\snort_output.log" -RedirectStandardError "$env:TEMP\snort_error.log" -Wait

    if (Test-Path "$env:TEMP\snort_output.log") {
        Get-Content "$env:TEMP\snort_output.log" | ForEach-Object { InfoMessage $_ }
        Remove-Item "$env:TEMP\snort_output.log" -Force
    }
    if (Test-Path "$env:TEMP\snort_error.log") {
        Get-Content "$env:TEMP\snort_error.log" | ForEach-Object { ErrorMessage $_ }
        Remove-Item "$env:TEMP\snort_error.log" -Force
    }

    if ($process.ExitCode -ne 0) {
        throw "Snort installation failed with exit code $($process.ExitCode)"
    }
}

function Uninstall-Snort {
    $SnortUrl = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-snort/refs/tags/v$WAZUH_SNORT_VERSION/scripts/uninstall.ps1"
    $UninstallSnortScript = "$env:TEMP\uninstall_snort.ps1"
    $global:InstallerFiles += $UninstallSnortScript
    $TaskName = "SnortStartup"

    $task = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
    if ($task) {
        InfoMessage "Removing existing Snort installation..."
        Invoke-WebRequest -Uri $SnortUrl -OutFile $UninstallSnortScript -ErrorAction Stop

        $process = Start-Process -FilePath "powershell.exe" -ArgumentList "-ExecutionPolicy Bypass -File `"$UninstallSnortScript`"" -NoNewWindow -PassThru -RedirectStandardOutput "$env:TEMP\uninstall_snort_output.log" -RedirectStandardError "$env:TEMP\uninstall_snort_error.log" -Wait

        if (Test-Path "$env:TEMP\uninstall_snort_output.log") {
            Get-Content "$env:TEMP\uninstall_snort_output.log" | ForEach-Object { InfoMessage $_ }
            Remove-Item "$env:TEMP\uninstall_snort_output.log" -Force
        }
        if (Test-Path "$env:TEMP\uninstall_snort_error.log") {
            Get-Content "$env:TEMP\uninstall_snort_error.log" | ForEach-Object { ErrorMessage $_ }
            Remove-Item "$env:TEMP\uninstall_snort_error.log" -Force
        }

        if ($process.ExitCode -ne 0) {
            WarningMessage "Snort uninstall completed with exit code $($process.ExitCode)"
        }
    }
}

function Install-Suricata {
    $SuricataUrl = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-suricata/refs/tags/v$WAZUH_SURICATA_VERSION/scripts/install.ps1"
    $SuricataScript = "$env:TEMP\suricata.ps1"
    $global:InstallerFiles += $SuricataScript

    InfoMessage "Downloading Suricata script..."
    Invoke-WebRequest -Uri $SuricataUrl -OutFile $SuricataScript -ErrorAction Stop
    InfoMessage "Installing Suricata..."

    $process = Start-Process -FilePath "powershell.exe" -ArgumentList "-ExecutionPolicy Bypass -File `"$SuricataScript`"" -NoNewWindow -PassThru -RedirectStandardOutput "$env:TEMP\suricata_output.log" -RedirectStandardError "$env:TEMP\suricata_error.log" -Wait

    if (Test-Path "$env:TEMP\suricata_output.log") {
        Get-Content "$env:TEMP\suricata_output.log" | ForEach-Object { InfoMessage $_ }
        Remove-Item "$env:TEMP\suricata_output.log" -Force
    }
    if (Test-Path "$env:TEMP\suricata_error.log") {
        Get-Content "$env:TEMP\suricata_error.log" | ForEach-Object { ErrorMessage $_ }
        Remove-Item "$env:TEMP\suricata_error.log" -Force
    }

    if ($process.ExitCode -ne 0) {
        throw "Suricata installation failed with exit code $($process.ExitCode)"
    }
}

function Uninstall-Suricata {
    $SuricataUrl = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-suricata/refs/tags/v$WAZUH_SURICATA_VERSION/scripts/uninstall.ps1"
    $UninstallSuricataScript = "$env:TEMP\uninstall_suricata.ps1"
    $global:InstallerFiles += $UninstallSuricataScript
    $TaskName = "SuricataStartup"

    $task = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
    if ($task) {
        InfoMessage "Removing existing Suricata installation..."
        Invoke-WebRequest -Uri $SuricataUrl -OutFile $UninstallSuricataScript -ErrorAction Stop

        $process = Start-Process -FilePath "powershell.exe" -ArgumentList "-ExecutionPolicy Bypass -File `"$UninstallSuricataScript`"" -NoNewWindow -PassThru -RedirectStandardOutput "$env:TEMP\uninstall_suricata_output.log" -RedirectStandardError "$env:TEMP\uninstall_suricata_error.log" -Wait

        if (Test-Path "$env:TEMP\uninstall_suricata_output.log") {
            Get-Content "$env:TEMP\uninstall_suricata_output.log" | ForEach-Object { InfoMessage $_ }
            Remove-Item "$env:TEMP\uninstall_suricata_output.log" -Force
        }
        if (Test-Path "$env:TEMP\uninstall_suricata_error.log") {
            Get-Content "$env:TEMP\uninstall_suricata_error.log" | ForEach-Object { ErrorMessage $_ }
            Remove-Item "$env:TEMP\uninstall_suricata_error.log" -Force
        }

        if ($process.ExitCode -ne 0) {
            WarningMessage "Suricata uninstall completed with exit code $($process.ExitCode)"
        }
    }
}

function Install-AgentStatus {
    $AgentStatusUrl = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/refs/heads/fix/agent-status-update-launcher/scripts/install.ps1"
    $AgentStatusScript = "$env:TEMP\install-agent-status.ps1"
    $global:InstallerFiles += $AgentStatusScript

    InfoMessage "Downloading Agent Status script..."
    Invoke-WebRequest -Uri $AgentStatusUrl -OutFile $AgentStatusScript -ErrorAction Stop
    InfoMessage "Installing Agent Status..."

    $process = Start-Process -FilePath "powershell.exe" -ArgumentList "-ExecutionPolicy Bypass -File `"$AgentStatusScript`"" -NoNewWindow -PassThru -RedirectStandardOutput "$env:TEMP\agentstatus_output.log" -RedirectStandardError "$env:TEMP\agentstatus_error.log" -Wait

    if (Test-Path "$env:TEMP\agentstatus_output.log") {
        Get-Content "$env:TEMP\agentstatus_output.log" | ForEach-Object { InfoMessage $_ }
        Remove-Item "$env:TEMP\agentstatus_output.log" -Force
    }
    if (Test-Path "$env:TEMP\agentstatus_error.log") {
        Get-Content "$env:TEMP\agentstatus_error.log" | ForEach-Object { ErrorMessage $_ }
        Remove-Item "$env:TEMP\agentstatus_error.log" -Force
    }

    if ($process.ExitCode -ne 0) {
        throw "Agent Status installation failed with exit code $($process.ExitCode)"
    }
}

function DownloadVersionFile {
    if (!(Test-Path -Path $OSSEC_PATH)) {
        WarningMessage "ossec-agent folder does not exist. Skipping version file."
    } else {
        InfoMessage "Downloading version file..."
        Invoke-WebRequest -Uri $VERSION_FILE_URL -OutFile $VERSION_FILE_PATH -ErrorAction Stop
        InfoMessage "Version file downloaded successfully"
    }
}

# ---- Main Installation Process ----
function Do-Install {
    $InstallBtn.Enabled = $false
    $NextBtn.Enabled = $false
    $SnortRadio.Enabled = $false
    $SuricataRadio.Enabled = $false
    $YaraCheckbox.Enabled = $false
    $ProgressBar.Value = 0
    $ProgressBar.Maximum = 100

    SectionSeparator "INSTALLATION START"

    try {
        # Calculate weights based on selected components
        $yaraWeight = if ($YaraCheckbox.Checked) { 15 } else { 0 }
        $nidsRemoveWeight = 10
        $nidsInstallWeight = 15
        
        # Calculate total weight and adjustment factor to reach 100%
        $totalWeight = 15 + 20 + 15 + $yaraWeight + $nidsRemoveWeight + $nidsInstallWeight + 10
        $weightFactor = 100 / $totalWeight
        
        # Adjust weights to ensure they sum to 100%
        $depsWeight = [math]::Round(15 * $weightFactor, 0)
        $agentWeight = [math]::Round(20 * $weightFactor, 0)
        $statusWeight = [math]::Round(15 * $weightFactor, 0)
        $yaraWeight = [math]::Round($yaraWeight * $weightFactor, 0)
        $nidsRemoveWeight = [math]::Round($nidsRemoveWeight * $weightFactor, 0)
        $nidsInstallWeight = [math]::Round($nidsInstallWeight * $weightFactor, 0)
        $versionWeight = 100 - ($depsWeight + $agentWeight + $statusWeight + $yaraWeight + $nidsRemoveWeight + $nidsInstallWeight)
        
        # Execute steps with adjusted weights
        Invoke-Step -Name "Installing Dependencies" -Weight $depsWeight -Action { Install-Dependencies }
        Invoke-Step -Name "Installing Wazuh Agent" -Weight $agentWeight -Action {
            # Use detected manager address if available, otherwise fall back to environment/default
            if ($global:DetectedManagerAddress) {
                Install-WazuhAgent -ManagerAddress $global:DetectedManagerAddress
            } else {
                Install-WazuhAgent
            }
        }
        Invoke-Step -Name "Installing Agent Status" -Weight $statusWeight -Action { Install-AgentStatus }
        
        if ($YaraCheckbox.Checked) {
            Invoke-Step -Name "Installing YARA" -Weight $yaraWeight -Action { Install-Yara }
        } else {
            Invoke-Step -Name "Removing YARA (if present)" -Weight $yaraWeight -Action { Uninstall-Yara }
        }

        # Install selected NIDS
        if ($SnortRadio.Checked) {
            Invoke-Step -Name "Removing Suricata (if present)" -Weight $nidsRemoveWeight -Action { Uninstall-Suricata }
            Invoke-Step -Name "Installing Snort" -Weight $nidsInstallWeight -Action { Install-Snort }
        } elseif ($SuricataRadio.Checked) {
            Invoke-Step -Name "Removing Snort (if present)" -Weight $nidsRemoveWeight -Action { Uninstall-Snort }
            Invoke-Step -Name "Installing Suricata" -Weight $nidsInstallWeight -Action { Install-Suricata }
        }

        Invoke-Step -Name "Downloading Version File" -Weight $versionWeight -Action { DownloadVersionFile }

        InfoMessage "Cleaning up installer files..."
        Cleanup-Installers
        SectionSeparator "INSTALLATION END"

        $global:InstallationComplete = $true
        SuccessMessage "Installation completed successfully! Click 'Next' to choose reboot option."

        # Enable Next button to proceed to reboot screen
        $NextBtn.Enabled = $true
        $InstallBtn.Enabled = $false

    } catch {
        [System.Windows.Forms.MessageBox]::Show("Installation failed: $($_.Exception.Message)","Installation Error",[System.Windows.Forms.MessageBoxButtons]::OK,[System.Windows.Forms.MessageBoxIcon]::Error) | Out-Null
        $InstallBtn.Enabled = $true
    } finally {
        $SnortRadio.Enabled = $true
        $SuricataRadio.Enabled = $true
        $YaraCheckbox.Enabled = $true
        $StatusLabel.Text = "Installation Complete"
    }
}

# ---- UI Creation ----
$form = New-Object System.Windows.Forms.Form
$form.Text = "Wazuh Agent Upgrade Assistant"
$form.Size = New-Object System.Drawing.Size(750,600)
$form.StartPosition = "CenterScreen"
$form.FormBorderStyle = "FixedDialog"
$form.MaximizeBox = $false

# Title Label
$Title = New-Object System.Windows.Forms.Label
$Title.Text = "Wazuh Agent Upgrade Assistant"
$Title.Font = New-Object System.Drawing.Font("Segoe UI",16,[System.Drawing.FontStyle]::Bold)
$Title.AutoSize = $true
$Title.Location = New-Object System.Drawing.Point(15,15)
$form.Controls.Add($Title)

$StatusLabel = New-Object System.Windows.Forms.Label
$StatusLabel.Text = "Ready"
$StatusLabel.AutoSize = $true
$StatusLabel.Location = New-Object System.Drawing.Point(18,55)
$form.Controls.Add($StatusLabel)

# Close button (top right)
$CloseBtn = New-Object System.Windows.Forms.Button
$CloseBtn.Text = "Close"
$CloseBtn.Size = New-Object System.Drawing.Size(80,30)
$CloseBtn.Location = New-Object System.Drawing.Point(640,15)
$CloseBtn.Add_Click({ $form.Close() })
$form.Controls.Add($CloseBtn)

# Log Box
$LogBox = New-Object System.Windows.Forms.TextBox
$LogBox.Multiline = $true
$LogBox.ReadOnly = $true
$LogBox.ScrollBars = "Vertical"
$LogBox.Font = New-Object System.Drawing.Font("Consolas",9)
$LogBox.Size = New-Object System.Drawing.Size(700,280)
$LogBox.Location = New-Object System.Drawing.Point(18,85)
$form.Controls.Add($LogBox)

# Progress Bar
$ProgressBar = New-Object System.Windows.Forms.ProgressBar
$ProgressBar.Size = New-Object System.Drawing.Size(700,22)
$ProgressBar.Location = New-Object System.Drawing.Point(18,375)
$form.Controls.Add($ProgressBar)

# Installation Panel
$InstallPanel = New-Object System.Windows.Forms.Panel
$InstallPanel.Size = New-Object System.Drawing.Size(700,140)
$InstallPanel.Location = New-Object System.Drawing.Point(18,405)
$form.Controls.Add($InstallPanel)

# NIDS Selection Group
$NidsGroup = New-Object System.Windows.Forms.GroupBox
$NidsGroup.Text = "Network IDS Selection"
$NidsGroup.Size = New-Object System.Drawing.Size(220,100)
$NidsGroup.Location = New-Object System.Drawing.Point(10,10)
$InstallPanel.Controls.Add($NidsGroup)

$SnortRadio = New-Object System.Windows.Forms.RadioButton
$SnortRadio.Text = "Install Snort"
$SnortRadio.Location = New-Object System.Drawing.Point(15,30)
$SnortRadio.AutoSize = $true
$NidsGroup.Controls.Add($SnortRadio)

$SuricataRadio = New-Object System.Windows.Forms.RadioButton
$SuricataRadio.Text = "Install Suricata"
$SuricataRadio.Location = New-Object System.Drawing.Point(15,60)
$SuricataRadio.AutoSize = $true
$SuricataRadio.Checked = $true
$NidsGroup.Controls.Add($SuricataRadio)

# Optional Components Group
$OptionalGroup = New-Object System.Windows.Forms.GroupBox
$OptionalGroup.Text = "Optional Components"
$OptionalGroup.Size = New-Object System.Drawing.Size(220,60)
$OptionalGroup.Location = New-Object System.Drawing.Point(240,10)
$InstallPanel.Controls.Add($OptionalGroup)

$YaraCheckbox = New-Object System.Windows.Forms.CheckBox
$YaraCheckbox.Text = "Install YARA"
$YaraCheckbox.Location = New-Object System.Drawing.Point(15,25)
$YaraCheckbox.AutoSize = $true
$YaraCheckbox.Checked = $false
$OptionalGroup.Controls.Add($YaraCheckbox)

$InstallBtn = New-Object System.Windows.Forms.Button
$InstallBtn.Text = "Start Installation"
$InstallBtn.Size = New-Object System.Drawing.Size(140,35)
$InstallBtn.Location = New-Object System.Drawing.Point(480,10)
$InstallBtn.Add_Click({ Do-Install })
$InstallPanel.Controls.Add($InstallBtn)

$OpenLogBtn = New-Object System.Windows.Forms.Button
$OpenLogBtn.Text = "Open Log"
$OpenLogBtn.Size = New-Object System.Drawing.Size(140,35)
$OpenLogBtn.Location = New-Object System.Drawing.Point(480,50)
$OpenLogBtn.Add_Click({
    if (Test-Path $LogPath) {
        Start-Process notepad.exe $LogPath
    } else {
        InfoMessage "No log file found at $LogPath"
    }
})
$InstallPanel.Controls.Add($OpenLogBtn)

$NextBtn = New-Object System.Windows.Forms.Button
$NextBtn.Text = "Next"
$NextBtn.Size = New-Object System.Drawing.Size(140,35)
$NextBtn.Location = New-Object System.Drawing.Point(480,90)
$NextBtn.Enabled = $false
$NextBtn.Add_Click({
    # Switch to reboot panel
    $InstallPanel.Visible = $false
    $RebootPanel.Visible = $true
    $Title.Text = "Installation Complete"
    $StatusLabel.Text = "Choose Reboot Option"
})
$InstallPanel.Controls.Add($NextBtn)

# Reboot Panel (initially hidden)
$RebootPanel = New-Object System.Windows.Forms.Panel
$RebootPanel.Size = New-Object System.Drawing.Size(700,140)
$RebootPanel.Location = New-Object System.Drawing.Point(18,405)
$RebootPanel.Visible = $false
$form.Controls.Add($RebootPanel)

$RebootMessage = New-Object System.Windows.Forms.Label
$RebootMessage.Text = "Installation completed successfully! A system reboot is recommended to ensure all changes take effect."
$RebootMessage.Size = New-Object System.Drawing.Size(680,40)
$RebootMessage.Location = New-Object System.Drawing.Point(10,10)
$RebootPanel.Controls.Add($RebootMessage)

$RebootNowBtn = New-Object System.Windows.Forms.Button
$RebootNowBtn.Text = "Reboot Now"
$RebootNowBtn.Size = New-Object System.Drawing.Size(150,40)
$RebootNowBtn.Location = New-Object System.Drawing.Point(200,70)
$RebootNowBtn.Add_Click({
    InfoMessage "Initiating system reboot..."
    $form.Close()
    Start-Process "shutdown.exe" -ArgumentList "/r /t 5 /c `"Rebooting to complete Wazuh Agent installation`"" -NoNewWindow
})
$RebootPanel.Controls.Add($RebootNowBtn)

$RebootLaterBtn = New-Object System.Windows.Forms.Button
$RebootLaterBtn.Text = "Reboot Later"
$RebootLaterBtn.Size = New-Object System.Drawing.Size(150,40)
$RebootLaterBtn.Location = New-Object System.Drawing.Point(370,70)
$RebootLaterBtn.Add_Click({
    InfoMessage "You chose to reboot later. Please restart your computer at your earliest convenience."
    $form.Close()
})
$RebootPanel.Controls.Add($RebootLaterBtn)

# ---- Startup Log ----
InfoMessage "Wazuh Agent Setup Wizard v2.0 (Alternative Version)"
InfoMessage "Running as Administrator: $IsAdmin"
InfoMessage "Log file: $LogPath"
InfoMessage "Wazuh Manager: $WAZUH_MANAGER"
InfoMessage "Agent Version: $WAZUH_AGENT_VERSION"
InfoMessage "Note: This version does not include OAuth2 client installation"

# Detect current Wazuh manager address
$global:DetectedManagerAddress = Get-CurrentWazuhManager
if ($global:DetectedManagerAddress) {
    InfoMessage "Current Manager in ossec.conf: $global:DetectedManagerAddress"
}

InfoMessage "Detecting existing IDS installations..."

# Detect and set default IDS
Set-DefaultIDS

InfoMessage "Detecting YARA installation..."

# Detect YARA and set checkbox state
if (Test-YaraInstalled) {
    $YaraCheckbox.Checked = $true
    InfoMessage "YARA is installed. Checkbox set to ON."
} else {
    $YaraCheckbox.Checked = $false
    InfoMessage "YARA is not installed. Checkbox set to OFF."
}

InfoMessage "Ready to install. Click 'Start Installation' to begin."

[void]$form.ShowDialog()
