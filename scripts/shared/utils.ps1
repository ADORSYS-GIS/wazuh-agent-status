# Centralized Utility Functions for Wazuh-Agent-Status PowerShell Scripts
# Designed to be downloaded and sourced via a bootstrap mechanism

# Function to handle logging with timestamp and optional colors
function Log {
    param (
        [Parameter(Mandatory)]
        [string]$Level,
        [Parameter(Mandatory)]
        [string]$Message,
        [string]$Color = "White"
    )
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Output "$Timestamp $Level $Message"
}

# Logging helpers
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

function ErrorExit {
    param ([string]$Message)
    ErrorMessage $Message
    exit 1
}

function PrintStep {
    param ([int]$StepNumber, [string]$Message)
    Log "[STEP]" "Step ${StepNumber}: $Message"
}

function Ensure-Directory {
    param (
        [Parameter(Mandatory)]
        [string]$Path
    )
    if (-not (Test-Path -Path $Path)) {
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
        InfoMessage "Created directory: $Path"
    }
}

function Get-FileChecksum {
    param([string]$FilePath)
    if (-not (Test-Path $FilePath)) {
        throw "File not found: $FilePath"
    }
    return (Get-FileHash -Path $FilePath -Algorithm SHA256).Hash.ToLower()
}

function Test-Checksum {
    param(
        [string]$FilePath,
        [string]$ExpectedHash
    )
    $actualHash = Get-FileChecksum -FilePath $FilePath
    if ($actualHash -ne $ExpectedHash.ToLower()) {
        ErrorMessage "Checksum verification FAILED for $FilePath!"
        ErrorMessage "  Expected: $ExpectedHash"
        ErrorMessage "  Got:      $actualHash"
        return $false
    }
    return $true
}

function Download-File {
    param(
        [string]$Url,
        [string]$Destination,
        [string]$Description = "file",
        [int]$MaxRetries = 3
    )

    InfoMessage "Downloading $Description..."

    $destDir = Split-Path -Parent $Destination
    if (-not (Test-Path $destDir)) {
        New-Item -ItemType Directory -Path $destDir -Force | Out-Null
    }

    $attempt = 0
    while ($attempt -lt $MaxRetries) {
        try {
            Invoke-WebRequest -Uri $Url -OutFile $Destination -UseBasicParsing
            SuccessMessage "$Description downloaded successfully"
            return
        } catch {
            $attempt++
            if ($attempt -lt $MaxRetries) {
                WarnMessage "Download failed, retrying ($attempt/$MaxRetries)..."
                Start-Sleep -Seconds 2
            }
        }
    }

    ErrorExit "Failed to download $Description from $Url after $MaxRetries attempts"
}

function Download-And-VerifyFile {
    param(
        [string]$Url,
        [string]$Destination,
        [string]$ChecksumPattern,
        [string]$FileName = "Unknown file",
        [string]$ChecksumFile = $global:ChecksumsPath,
        [string]$ChecksumUrl = $global:ChecksumsURL
    )

    Download-File -Url $Url -Destination $Destination -Description $FileName

    # If a direct checksum URL is provided, download it and use it as the source of truth
    $finalChecksumFile = $ChecksumFile
    $isTempFile = $false
    if (-not [string]::IsNullOrWhiteSpace($ChecksumUrl)) {
        $finalChecksumFile = Join-Path ([System.IO.Path]::GetTempPath()) "checksums-$([System.Guid]::NewGuid().ToString()).sha256"
        Download-File -Url $ChecksumUrl -Destination $finalChecksumFile -Description "checksum file"
        $isTempFile = $true
    }

    try {
        if ([string]::IsNullOrWhiteSpace($finalChecksumFile) -or -not (Test-Path -Path $finalChecksumFile)) {
            ErrorExit "Checksum file not found at $finalChecksumFile, cannot verify $FileName"
        }

        $matchingLine = Select-String -Path $finalChecksumFile -Pattern $ChecksumPattern | Select-Object -First 1
        if ($null -eq $matchingLine) {
            ErrorExit "No checksum found for $FileName in $finalChecksumFile using pattern $ChecksumPattern"
        }

        $expectedHash = $matchingLine.Line.Split(" ")[0].Trim()
        if ([string]::IsNullOrWhiteSpace($expectedHash)) {
            ErrorExit "No checksum found for $FileName in $finalChecksumFile using pattern $ChecksumPattern (empty hash extracted)"
        }

        if (-not (Test-Checksum -FilePath $Destination -ExpectedHash $expectedHash)) {
            ErrorExit "$FileName checksum verification failed"
        }
        InfoMessage "$FileName checksum verification passed."
    } finally {
        # Cleanup temporary checksum file if it was created
        if ($isTempFile -and (Test-Path -Path $finalChecksumFile)) {
            Remove-Item -Path $finalChecksumFile -Force -ErrorAction SilentlyContinue
        }
    }

    SuccessMessage "$FileName downloaded and verified successfully."
    return $true
}

# Ensure the script is running with administrator privileges
function EnsureAdmin {
    if (-not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
        ErrorExit "This script requires administrative privileges. Please run it as Administrator."
    }
}