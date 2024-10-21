# Set default application details
$APP_NAME = $env:APP_NAME -or "wazuh-agent-status"
$WOPS_VERSION = $env:WOPS_VERSION -or "0.1.2"

# Define text formatting (Windows doesn't support color in native console, this is a placeholder)
$RED = "RED"
$GREEN = "GREEN"
$YELLOW = "YELLOW"
$BLUE = "BLUE"
$BOLD = ""
$NORMAL = ""

# Function for logging with timestamp
function Log {
    param(
        [string]$LEVEL,
        [string]$MESSAGE
    )
    $TIMESTAMP = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "$TIMESTAMP $LEVEL $MESSAGE"
}

# Logging helpers
function Info-Message {
    param(
        [string]$Message
    )
    Log "$BLUE$BOLD[INFO]$NORMAL" $Message
}

function Warn-Message {
    param(
        [string]$Message
    )
    Log "$YELLOW$BOLD[WARNING]$NORMAL" $Message
}

function Error-Message {
    param(
        [string]$Message
    )
    Log "$RED$BOLD[ERROR]$NORMAL" $Message
}

function Success-Message {
    param(
        [string]$Message
    )
    Log "$GREEN$BOLD[SUCCESS]$NORMAL" $Message
}

function Print-Step {
    param(
        [int]$StepNumber,
        [string]$Message
    )
    Log "$BLUE$BOLD[STEP]$NORMAL" "$StepNumber: $Message"
}

# Exit script with an error message
function Error-Exit {
    param(
        [string]$Message
    )
    Error-Message $Message
    exit 1
}

# Check if a command exists
function Command-Exists {
    param(
        [string]$Command
    )
    Get-Command $Command -ErrorAction SilentlyContinue
}

# Ensure admin privileges
function EnsureAdmin {
    if (-Not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
        ErrorExit "This script requires administrative privileges. Please run it as Administrator."
    }
}

# Ensure user and group (Windows equivalent is ensuring local user or group exists)
function EnsureUserGroup {
    InfoMessage "Ensuring that the ${USER}:${GROUP} user and group exist..."

    if (-Not (Get-LocalUser -Name $USER -ErrorAction SilentlyContinue)) {
        InfoMessage "Creating user $USER..."
        New-LocalUser -Name $USER -NoPassword
    }

    if (-Not (Get-LocalGroup -Name $GROUP -ErrorAction SilentlyContinue)) {
        InfoMessage "Creating group $GROUP..."
        New-LocalGroup -Name $GROUP
    }
}

# Determine architecture and operating system
$OS = if ($PSVersionTable.PSEdition -eq "Core") { "linux" } else { "windows" }
$ARCH = if ([Environment]::Is64BitOperatingSystem) { "amd64" } else { "amd32" }

if ($OS -ne "windows") {
    ErrorExit "Unsupported operating system: $OS"
}

if ($ARCH -ne "amd64" -and $ARCH -ne "amd32") {
    ErrorExit "Unsupported architecture: $ARCH"
}
# Construct binary name and URL for download
$BIN_NAME = "$APP_NAME-$OS-$ARCH"
$BASE_URL = "https://github.com/ADORSYS-GIS/$APP_NAME/releases/download/v$WOPS_VERSION"
$URL = "$BASE_URL/$BIN_NAME.exe"

# Fallback URL if the constructed URL fails
$FALLBACK_URL = "https://github.com/ADORSYS-GIS/wazuh-agent-status/releases/download/v0.1.2/wazuh-agent-status-windows-amd64.exe"


# Step 1: Download the binary file
$TEMP_FILE = New-TemporaryFile
PrintStep 1 "Downloading $BIN_NAME from $URL..."
try {
    Invoke-WebRequest -Uri $URL -OutFile $TEMP_FILE -UseBasicParsing -ErrorAction Stop
} catch {
    WarnMessage "Failed to download from $URL. Trying fallback URL..."
    Invoke-WebRequest -Uri $FALLBACK_URL -OutFile $TEMP_FILE -UseBasicParsing -ErrorAction Stop
}

# Step 2: Install the binary based on architecture
if ($ARCH -eq "amd64") {
    $BIN_DIR = "C:\Program Files (x86)\ossec-agent"
} else {
    $BIN_DIR = "C:\Program Files\ossec-agent"
}

PrintStep 2 "Installing binary to $BIN_DIR..."
New-Item -ItemType Directory -Path $BIN_DIR -Force
Move-Item -Path $TEMP_FILE -Destination "$BIN_DIR\$APP_NAME.exe"
icacls "$BIN_DIR\$APP_NAME.exe" /grant Users:RX



Success-Message "Installation and configuration complete! You can now use '$APP_NAME' from your terminal."
Info-Message "Run ``& '$BIN_DIR\$APP_NAME.exe'`` to start configuring."
