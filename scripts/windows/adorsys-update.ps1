#requires -version 5.1

# ---- Parameters ----
param(
    [switch]$Prerelease,
    [switch]$Update
)

# Set strict mode for error handling
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ---- Elevate ----
$IsAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $IsAdmin) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName  = (Get-Process -Id $PID).Path
    $arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$($MyInvocation.MyCommand.Path)`""

    if ($Prerelease) {
        $arguments += " -Prerelease"
    }
    if ($Update) {
        $arguments += " -Update"
    }

    $psi.Arguments = $arguments
    $psi.Verb      = "runas"
    try {
        [System.Diagnostics.Process]::Start($psi) | Out-Null
        exit
    } catch {
        Write-Host "Administrator approval is required. Exiting."
        exit 1
    }
}

# Configuration

$APP_VERSION = if ($env:APP_VERSION) { $env:APP_VERSION } else { "0.5.2" }
$REPO_REF = if ($env:WAZUH_AGENT_STATUS_REPO_REF) { $env:WAZUH_AGENT_STATUS_REPO_REF } else { "main" }
$REPO_URL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/$REPO_REF"
$AGENT_REPO_REF = if ($env:WAZUH_AGENT_REPO_REF) { $env:WAZUH_AGENT_REPO_REF } else { "main" }
$TMP = Join-Path $env:TEMP "wazuh-agent-status-install"; if (-not (Test-Path $TMP)) { mkdir $TMP | Out-Null }

try {
    $global:ChecksumsPath = Join-Path $TMP "checksums.sha256"; $U = Join-Path $TMP "utils.ps1"
    Invoke-WebRequest "$REPO_URL/checksums.sha256" -OutFile $global:ChecksumsPath; Invoke-WebRequest "$REPO_URL/scripts/shared/utils.ps1" -OutFile $U
    if ((Get-FileHash $U -Alg SHA256).Hash.ToLower() -ne (Select-String -Path $global:ChecksumsPath -Pattern "scripts/shared/utils.ps1").Line.Split(" ")[0].ToLower()) { throw }
    . $U
} catch { Write-Error "Bootstrap failed"; exit 1 }

EnsureWindows
EnsureAdmin

# Cleanup bootstrap files on exit
Register-EngineEvent -SourceIdentifier ([System.Guid]::NewGuid().ToString()) -Action {
    Remove-Item -Path $TMP -Recurse -Force -ErrorAction SilentlyContinue
} | Out-Null

# ---- Configuration Variables ----
$CURRENT_MANAGER = $null
$OssecConfPath = "C:\Program Files (x86)\ossec-agent\ossec.conf"
if (Test-Path $OssecConfPath) {
    $addressLine = Select-String -Path $OssecConfPath -Pattern "<address>(.*?)</address>" | Select-Object -First 1
    if ($addressLine -match "<address>(.*?)</address>") {
        $CURRENT_MANAGER = $matches[1]
    }
}
$WAZUH_MANAGER           = if ($env:WAZUH_MANAGER) { $env:WAZUH_MANAGER } elseif ($CURRENT_MANAGER) { $CURRENT_MANAGER } else { "wazuh.example.com" }
$OSSEC_PATH              = "C:\Program Files (x86)\ossec-agent\"
$VERSION_URL             = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/$AGENT_REPO_REF/versions.json"
$STABLE_SETUP_SCRIPT_URL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/$AGENT_REPO_REF/scripts/windows/setup-agent.ps1"

# ---- Globals ----
$ActiveResponsesLogDir = Join-Path $OSSEC_PATH "active-response"
$LogPath               = Join-Path $ActiveResponsesLogDir "active-responses.log"
$PRERELEASE_VERSION    = $null

# ---- Logging Override ----
function Append-Log {
    param(
        [string]$Message,
        [string]$Level = "INFO"
    )

    $ts   = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
    $line = "[$ts] [$Level] $Message"

    try {
        if (-not (Test-Path $ActiveResponsesLogDir)) {
            New-Item -ItemType Directory -Force -Path $ActiveResponsesLogDir -ErrorAction Stop | Out-Null
        }

        $fileStream   = $null
        $streamWriter = $null
        try {
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
            if ($fileStream)   { $fileStream.Dispose() }
        }
    } catch {
        # Fallback to standard host output if log file writing fails
        Write-Output "Warning: Failed to write to log file $LogPath : $($_.Exception.Message)"
    }

    Write-Output $line
}

# ---- Helper: clean up a temp file unconditionally ----
function Remove-TempFile {
    param([string]$Path)
    if (Test-Path $Path) {
        Remove-Item $Path -Force -ErrorAction SilentlyContinue
    }
}

function Get-ActiveConsoleUser {
    # 1. Query process owner of explorer.exe (works reliably from Session 0 / background service)
    try {
        $explorer = Get-CimInstance Win32_Process -Filter "Name = 'explorer.exe'" -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($explorer) {
            $owner = Invoke-CimMethod -InputObject $explorer -MethodName GetOwner -ErrorAction SilentlyContinue
            if ($owner -and $owner.User) {
                if ($owner.Domain) { return "$($owner.Domain)\$($owner.User)" }
                return $owner.User
            }
        }
    } catch {}

    # 2. Query quser for active session user
    try {
        $quser = query user 2>$null
        if ($quser) {
            foreach ($line in ($quser -split "\r?\n")[1..($quser.Count-1)]) {
                if ($line -match "Active") {
                    $fields = $line.Trim() -split "\s+"
                    $u = $fields[0].Replace(">", "")
                    if ($u) { return $u }
                }
            }
        }
    } catch {}

    # 3. Fallback to Win32_ComputerSystem
    try {
        $sysUser = (Get-CimInstance -ClassName Win32_ComputerSystem -ErrorAction SilentlyContinue).UserName
        if (-not [string]::IsNullOrWhiteSpace($sysUser)) { return $sysUser }
    } catch {}

    return $null
}

function Invoke-InteractivePopup {
    param(
        [string]$Message,
        [string]$Title = "Wazuh Update",
        [string]$Buttons = "OK",
        [string]$Icon = "Information",
        [int]$TimeoutSeconds = 600
    )
    
    try {
        $consoleUser = Get-ActiveConsoleUser
        if ([string]::IsNullOrWhiteSpace($consoleUser)) {
            InfoMessage "No interactive user session found. Defaulting to proceed."
            return 0
        }
        
        $guid = [guid]::NewGuid().ToString('N')
        $taskName = "WazuhUpdatePopup_$guid"
        $resultFile = Join-Path $env:TEMP "wazuh_popup_res_$guid.txt"

        $escapedMsg = $Message.Replace("'", "''")
        $escapedTitle = $Title.Replace("'", "''")
        
        $script = @"
Add-Type -AssemblyName System.Windows.Forms
`$res = [System.Windows.Forms.MessageBox]::Show('$escapedMsg', '$escapedTitle', '$Buttons', '$Icon')
Set-Content -Path '$resultFile' -Value `$res.ToString() -Force
"@
        $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($script))
        
        $action = New-ScheduledTaskAction -Execute "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe" -Argument "-NoProfile -WindowStyle Hidden -EncodedCommand $encoded"
        $principal = New-ScheduledTaskPrincipal -UserId $consoleUser -LogonType Interactive
        
        Register-ScheduledTask -TaskName $taskName -Action $action -Principal $principal -Force | Out-Null
        Start-ScheduledTask -TaskName $taskName | Out-Null
        
        $elapsed = 0
        while (-not (Test-Path $resultFile)) {
            if ($elapsed -ge $TimeoutSeconds) {
                WarnMessage "Popup timed out after $TimeoutSeconds seconds."
                Stop-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue | Out-Null
                Unregister-ScheduledTask -TaskName $taskName -Confirm:$false | Out-Null
                Remove-TempFile $resultFile
                return 1
            }
            Start-Sleep -Seconds 1
            $elapsed++
        }
        
        $userChoice = (Get-Content -Path $resultFile -Raw).Trim()
        Unregister-ScheduledTask -TaskName $taskName -Confirm:$false | Out-Null
        Remove-TempFile $resultFile
        
        if ($userChoice -eq "Yes" -or $userChoice -eq "OK") {
            return 0
        } else {
            return 1
        }
    } catch {
        WarnMessage "Interactive popup failed: $($_.Exception.Message)"
        return 0
    }
}

function Get-UserConsent {
    InfoMessage "Prompting user for upgrade consent..."
    $msg = "A new version of Wazuh is available. Would you like to upgrade?"
    $res = Invoke-InteractivePopup -Message $msg -Buttons "YesNo" -Icon "Question" -TimeoutSeconds 3600
    if ($res -ne 0) {
        InfoMessage "User chose Remind Me Later or prompt timed out."
        return $false
    }
    InfoMessage "User chose to Upgrade Now."
    return $true
}

function Get-PrereleaseVersion {
    try {
        InfoMessage "Fetching prerelease version from: $VERSION_URL"
        $response = Invoke-RestMethod -Uri $VERSION_URL -Method Get -TimeoutSec 30

        if ($response -and $response.framework -and $response.framework.prerelease_version) {
            $version = $response.framework.prerelease_version
            InfoMessage "Successfully fetched prerelease version: $version"
            return $version
        } else {
            WarnMessage "No prerelease version found in response."
            return $null
        }
    } catch {
        WarnMessage "Failed to fetch prerelease version: $($_.Exception.Message)"
        return $null
    }
}

function Run-Update {
    InfoMessage "Starting Wazuh agent upgrade..."
    InfoMessage "Using temporary directory: $env:TEMP"

    # Determine setup script URL without shadowing the module-level constant
    if ($Prerelease) {
        $resolvedScriptUrl = $PRERELEASE_SETUP_SCRIPT_URL
        InfoMessage "Using prerelease setup script: $resolvedScriptUrl"
    } else {
        $resolvedScriptUrl = $STABLE_SETUP_SCRIPT_URL
        InfoMessage "Using stable setup script: $resolvedScriptUrl"
    }

    $setupScriptPath = Join-Path $env:TEMP "setup-agent.ps1"

    InfoMessage "Downloading setup script..."
    try {
        Invoke-WebRequest -Uri $resolvedScriptUrl -OutFile $setupScriptPath -ErrorAction Stop
    } catch {
        ErrorMessage "Failed to download setup-agent.ps1: $($_.Exception.Message)"
        exit 1
    }

    InfoMessage "Executing setup script: $setupScriptPath"
    $env:WAZUH_MANAGER = $WAZUH_MANAGER
    # Tell install.ps1 this is an update so it leaves the running server
    # service (the one executing this chain) alone instead of stopping it.
    $env:WAZUH_AGENT_STATUS_UPDATE = "1"
    try {
        & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $setupScriptPath
        if ($LASTEXITCODE -ne 0) {
            ErrorMessage "Setup script failed (exit code: $LASTEXITCODE)."
            exit 1
        }
    } catch {
        ErrorMessage "Failed to execute setup script: $($_.Exception.Message)"
        exit 1
    } finally {
        Remove-TempFile $setupScriptPath
    }

    SuccessMessage "Update completed successfully! Please save your work and reboot to finish the update."
    # Fire and forget success popup (timeout short so it doesn't block indefinitely)
    Invoke-InteractivePopup -Message "Update completed successfully! Please save your work and reboot to finish the update." -Buttons "OK" -Icon "Information" -TimeoutSeconds 60 | Out-Null
}

# ---- Main Execution ----
InfoMessage "Wazuh Agent Upgrade Script"
InfoMessage "Running as Administrator: $IsAdmin"
InfoMessage "Log file: $LogPath"

if (-not (Get-UserConsent)) {
    InfoMessage "Update postponed. Exiting."
    exit 0
}

if ($Prerelease) {
    $PRERELEASE_VERSION = Get-PrereleaseVersion
    if ($PRERELEASE_VERSION) {
        InfoMessage "PRERELEASE UPGRADE MODE: Installing prerelease version $PRERELEASE_VERSION"
        $PRERELEASE_SETUP_SCRIPT_URL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/tags/v$PRERELEASE_VERSION/scripts/windows/setup-agent.ps1"
        # Point setup-agent.ps1 at the same tag so it downloads its components
        # and version.txt from the prerelease release.
        $env:WAZUH_AGENT_REPO_REF = "refs/tags/v$PRERELEASE_VERSION"
    } else {
        WarnMessage "Failed to fetch prerelease version. Exiting."
        exit 1
    }
} else {
    InfoMessage "STABLE UPGRADE MODE: Installing latest stable version."
}

InfoMessage "Starting upgrade process..."
Run-Update
InfoMessage "Script execution completed."
