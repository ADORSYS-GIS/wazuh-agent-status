# Define variables used in the uninstallation script
$SERVER_NAME =  "wazuh-agent-status" 
$CLIENT_NAME =  "wazuh-agent-status-client"
$BIN_DIR = "C:\Program Files\$SERVER_NAME"
$SERVER_EXE = "$BIN_DIR\$SERVER_NAME.exe"
$CLIENT_EXE = "$BIN_DIR\$CLIENT_NAME.exe"


function Log {
    param (
        [string]$Level,
        [string]$Message,
        [string]$Color = "White"  # Default color
    )
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "$Timestamp $Level $Message" -ForegroundColor $Color
}

# Logging helpers with colors
function InfoMessage {
    param ([string]$Message)
    Log "[INFO]" $Message "White"
}

function WarnMessage {
    param ([string]$Message)
    Log "[WARNING]" $Message "Yellow"
}

function ErrorMessage {
    param ([string]$Message)
    Log "[ERROR]" $Message "Red"
}

function SuccessMessage {
    param ([string]$Message)
    Log "[SUCCESS]" $Message "Green"
}

function PrintStep {
    param (
        [int]$StepNumber,
        [string]$Message
    )
    Log "[STEP]" "Step ${StepNumber}: $Message" "White"
}

# Exit script with an error message
function ErrorExit {
    param ([string]$Message)
    ErrorMessage $Message
    exit 1
}


function Remove-File {
    param (
        [Parameter(Mandatory = $true)]
        [string]$FilePath
    )
    InfoMessage "Removing '$FilePath'"
    try {
        if (Test-Path -Path $FilePath) {
            Remove-Item -Path $FilePath -Force -ErrorAction Stop
            InfoMessage "File '$FilePath' has been successfully removed." 
        } else {
            WarnMessage "File '$FilePath' does not exist."
        }
    } catch {
        ErrorMessage "An error occurred while trying to remove the file: $_"
    }
}


function Remove-Service {
    param (
        [Parameter(Mandatory=$true)]
        [string]$ServiceName
    )

    # Check if the service exists
    $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue

    if ($service) {
        # Stop the service if it's running
        if ($service.Status -eq 'Running') {
            
            Stop-Service -Name $ServiceName -Force
        }

        # Remove the service using sc.exe
        sc.exe delete $ServiceName | Out-Null

        InfoMessage "Service '$ServiceName' has been removed successfully."
    } else {
        WarnMessage "Service '$ServiceName' not found."
    }
}


function Remove-StartupShortcut {
    param (
        [Parameter(Mandatory = $true)]
        [string]$ShortcutName
    )


    # Check if the process is running
    $process = Get-Process -Name $ShortcutName -ErrorAction SilentlyContinue

    if ($process) {
        InfoMessage "Process '$ShortcutName' is running. Stopping it..."
        Stop-Process -Name $ShortcutName -Force
        InfoMessage "Process '$ShortcutName' has been stopped."
    }
    else {
        WarnMessage "Process '$ShortcutName' is not running. Skipping..."
    }
    # Define full path of the shortcut

    InfoMessage "Removing Shortcut '$ShortcutName' from Startup..."
    $ShortcutPath = [System.IO.Path]::Combine($env:APPDATA, "Microsoft\Windows\Start Menu\Programs\Startup", "$ShortcutName.lnk")
    
    # Check if the shortcut exists and remove it
    if (Test-Path $ShortcutPath) {
        Remove-Item -Path $ShortcutPath -Force
        InfoMessage "Shortcut '$ShortcutName' removed from Startup."
    } else {
        WarnMessage "Shortcut '$ShortcutName' not found in Startup."
    }
}


function Remove-Binaries {
    Remove-File $SERVER_EXE
    Remove-File $CLIENT_EXE
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