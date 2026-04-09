#requires -version 5.1

# ---- Parameters ----
param(
    [switch]$Prerelease,
    [switch]$Update
)

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
$APP_VERSION = if ($null -ne $env:APP_VERSION) { $env:APP_VERSION } else { "0.4.2.rc1" }

if ($INSTALL_PROFILE -eq "admin") {
    $WAS_VERSION = $APP_VERSION
} else {
    $WAS_VERSION = "$APP_VERSION-user"
}
$WAZUH_AGENT_STATUS_REPO_REF = if ($null -ne $env:WAZUH_AGENT_STATUS_REPO_REF) { $env:WAZUH_AGENT_STATUS_REPO_REF } else { "refs/tags/v$WAS_VERSION" }
$WAZUH_AGENT_STATUS_REPO_URL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/$WAZUH_AGENT_STATUS_REPO_REF"

$TEMP_DIR = Join-Path $env:TEMP "wazuh-agent-status-install"
if (-not (Test-Path $TEMP_DIR)) {
    New-Item -Path $TEMP_DIR -ItemType Directory | Out-Null
}

try {
    $ChecksumsURL = "$WAZUH_AGENT_STATUS_REPO_URL/checksums.sha256"
    $UtilsURL = "$WAZUH_AGENT_STATUS_REPO_URL/scripts/shared/utils.ps1"
    
    $global:ChecksumsPath = Join-Path $TEMP_DIR "checksums.sha256"
    $UtilsPath = Join-Path $TEMP_DIR "utils.ps1"

    Invoke-WebRequest -Uri $ChecksumsURL -OutFile $ChecksumsPath -ErrorAction Stop
    Invoke-WebRequest -Uri $UtilsURL -OutFile $UtilsPath -ErrorAction Stop

    # Verification function (bootstrap)
    function Get-FileChecksum-Bootstrap {
        param([string]$FilePath)
        return (Get-FileHash -Path $FilePath -Algorithm SHA256).Hash.ToLower()
    }

    $ExpectedHash = (Select-String -Path $ChecksumsPath -Pattern "scripts/shared/utils.ps1").Line.Split(" ")[0]
    $ActualHash = Get-FileChecksum-Bootstrap -FilePath $UtilsPath

    if ([string]::IsNullOrWhiteSpace($ExpectedHash) -or ($ActualHash -ne $ExpectedHash.ToLower())) {
        Write-Error "Checksum verification failed for utils.ps1"
        Write-Error "Expected: $ExpectedHash"
        Write-Error "Got:      $ActualHash"
        exit 1
    }

    . $UtilsPath
}
catch {
    Write-Error "Failed to initialize utilities: $($_.Exception.Message)"
    exit 1
}

# Cleanup bootstrap files on exit
Register-EngineEvent -SourceIdentifier ([System.Guid]::NewGuid().ToString()) -Action {
    Remove-Item -Path $TMP_DIR -Recurse -Force -ErrorAction SilentlyContinue
} | Out-Null

Set-StrictMode -Version Latest

# ---- Configuration Variables ----
$WAZUH_MANAGER           = if ($env:WAZUH_MANAGER) { $env:WAZUH_MANAGER } else { "wazuh.example.com" }
$OSSEC_PATH              = "C:\Program Files (x86)\ossec-agent\"
$WAZUH_AGENT_REPO_REF    = if ($env:WAZUH_AGENT_REPO_REF) { $env:WAZUH_AGENT_REPO_REF } else { "main" }
$VERSION_URL             = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/${WAZUH_AGENT_REPO_REF}/versions.json"
$STABLE_SETUP_SCRIPT_URL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/${WAZUH_AGENT_REPO_REF}/scripts/windows/setup-agent.ps1"

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
        # Silently ignore log-write errors so they don't mask real failures
    }

    Write-Host $line
}

# ---- Helper: clean up a temp file unconditionally ----
function Remove-TempFile {
    param([string]$Path)
    if (Test-Path $Path) {
        Remove-Item $Path -Force -ErrorAction SilentlyContinue
    }
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
            WarningMessage "No prerelease version found in response."
            return $null
        }
    } catch {
        WarningMessage "Failed to fetch prerelease version: $($_.Exception.Message)"
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
    $stdoutLog       = Join-Path $env:TEMP "setup_output.log"
    $stderrLog       = Join-Path $env:TEMP "setup_error.log"

    InfoMessage "Downloading setup script..."
    try {
        Invoke-WebRequest -Uri $resolvedScriptUrl -OutFile $setupScriptPath -ErrorAction Stop
    } catch {
        ErrorMessage "Failed to download setup-agent.ps1: $($_.Exception.Message)"
        exit 1
    }

    # Always pass -Upgrade to the downloaded setup script; add -Prerelease when in prerelease mode
    $setupArgs = @("-ExecutionPolicy", "Bypass", "-File", "`"$setupScriptPath`"", "-Update")

    $flagSummary = ($setupArgs | Where-Object { $_ -like '-*' } | Select-Object -Skip 2) -join " "
    InfoMessage "Executing setup script with flags: $flagSummary"

    try {
        $process = Start-Process `
            -FilePath "powershell.exe" `
            -ArgumentList $setupArgs `
            -NoNewWindow `
            -PassThru `
            -RedirectStandardOutput $stdoutLog `
            -RedirectStandardError  $stderrLog `
            -Wait

        # Flush logs before checking exit code
        if (Test-Path $stdoutLog) {
            Get-Content $stdoutLog | ForEach-Object { InfoMessage $_ }
        }
        if (Test-Path $stderrLog) {
            Get-Content $stderrLog | ForEach-Object { ErrorMessage $_ }
        }

        if ($process.ExitCode -ne 0) {
            ErrorMessage "Setup script failed (exit code: $($process.ExitCode))."
            exit 1
        }
    } catch {
        ErrorMessage "Failed to execute setup script: $($_.Exception.Message)"
        exit 1
    } finally {
        # Clean up temp files in ALL code paths (success, failure, exception)
        Remove-TempFile $stdoutLog
        Remove-TempFile $stderrLog
        Remove-TempFile $setupScriptPath
    }

    SuccessMessage "Update completed successfully! Please save your work and reboot to finish the update."
}

# ---- Main Execution ----
InfoMessage "Wazuh Agent Upgrade Script"
InfoMessage "Running as Administrator: $IsAdmin"
InfoMessage "Log file: $LogPath"

# Resolve prerelease version here — after all functions are defined
if ($Prerelease) {
    $PRERELEASE_VERSION = Get-PrereleaseVersion
    if ($PRERELEASE_VERSION) {
        InfoMessage "PRERELEASE UPGRADE MODE: Installing prerelease version $PRERELEASE_VERSION"
    } else {
        WarningMessage "Failed to fetch prerelease version. Exiting."
        exit 1
    }
} else {
    InfoMessage "STABLE UPGRADE MODE: Installing latest stable version."
}

InfoMessage "Starting upgrade process..."
Run-Update
InfoMessage "Script execution completed."