# Configuration
$APP_VERSION = if ($null -ne $env:APP_VERSION) { $env:APP_VERSION } else { "0.4.2.rc1" }

if ($INSTALL_PROFILE -eq "admin") {
    $WAS_VERSION = $APP_VERSION
} else {
    $WAS_VERSION = "$APP_VERSION-user"
}
$WAZUH_AGENT_STATUS_REPO_REF = if ($null -ne $env:WAZUH_AGENT_STATUS_REPO_REF) { $env:WAZUH_AGENT_STATUS_REPO_REF } else { "refs/tags/v$WAS_VERSION" }
$WAZUH_AGENT_STATUS_REPO_URL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/$WAZUH_AGENT_STATUS_REPO_REF"

$TMP_DIR = Join-Path $env:TEMP "wazuh-agent-status-install"
if (-not (Test-Path $TMP_DIR)) {
    New-Item -Path $TMP_DIR -ItemType Directory | Out-Null
}

try {
    $ChecksumsURL = "$WAZUH_AGENT_STATUS_REPO_URL/checksums.sha256"
    $UtilsURL = "$WAZUH_AGENT_STATUS_REPO_URL/scripts/shared/utils.ps1"
    
    $global:ChecksumsPath = Join-Path $TMP_DIR "checksums.sha256"
    $UtilsPath = Join-Path $TMP_DIR "utils.ps1"

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

# Environment Variables with Defaults
$SERVER_NAME = if ($null -ne $env:SERVER_NAME) { $env:SERVER_NAME } else { "wazuh-agent-status" }
$CLIENT_NAME = if ($null -ne $env:CLIENT_NAME) { $env:CLIENT_NAME } else { "wazuh-agent-status-client" }
$BIN_DIR = "C:\Program Files\$SERVER_NAME"
$SERVER_EXE = "$BIN_DIR\$SERVER_NAME.exe"
$CLIENT_EXE = "$BIN_DIR\$CLIENT_NAME.exe"
$UPDATE_SCRIPT_PATH = "${env:ProgramFiles(x86)}\ossec-agent\active-response\bin\adorsys-update.bat"



function Remove-Binaries {
    Remove-File $SERVER_EXE
    Remove-File $CLIENT_EXE
    Remove-File $UPDATE_SCRIPT_PATH
    Remove-File $BIN_DIR
}

# Function to uninstall application and clean up
function Uninstall-WazuhAgentStatus {
    try {

        Remove-StartupShortcut -ShortcutName $CLIENT_NAME
        Remove-Service -ServiceName $SERVER_NAME

        Remove-Binaries
        SuccessMessage "Wazuh Agent Status uninstalled successfully"
    }
    catch {
        ErrorMessage "Wazuh Agent Status Uninstall Failed: $($_.Exception.Message)"
    }
}

Uninstall-WazuhAgentStatus