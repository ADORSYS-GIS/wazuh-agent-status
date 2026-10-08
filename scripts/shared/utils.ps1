# Centralized Utility Functions for Wazuh-Agent-Status PowerShell Scripts
# Designed to be downloaded and sourced via a bootstrap mechanism

# PSScriptAnalyzer rule suppressions for style rules intentionally not applied
# to this installer utility library (see ticket #251):
#   - PSUseApprovedVerbs: helper names (Ensure-Directory, Prepare-DestinationFile,
#     Download-File, Download-And-VerifyFile) use verbs with clear intent for
#     installer code; no approved verb matches "Ensure"/"Download"/"Prepare"
#     without an awkward rename.
[Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSUseApprovedVerbs', '')]
param()

# Function to handle logging with timestamp
function Log {
    param (
        [Parameter(Mandatory)]
        [string]$Level,
        [Parameter(Mandatory)]
        [AllowEmptyString()]
        [string]$Message
    )
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Output "$Timestamp $Level $Message"
}

# Logging helpers
function InfoMessage {
    param ([string]$Message)
    Log -Level "[INFO]" -Message $Message
}

function WarnMessage {
    param ([string]$Message)
    Log -Level "[WARNING]" -Message $Message
}

function ErrorMessage {
    param ([string]$Message)
    Log -Level "[ERROR]" -Message $Message
}

function SuccessMessage {
    param ([string]$Message)
    Log -Level "[SUCCESS]" -Message $Message
}

function ErrorExit {
    param ([string]$Message)
    ErrorMessage -Message $Message
    exit 1
}

function PrintStep {
    param ([int]$StepNumber, [string]$Message)
    Log -Level "[STEP]" -Message "Step ${StepNumber}: $Message"
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

function Prepare-DestinationFile {
    param([string]$Destination)
    if (Test-Path -LiteralPath $Destination) {
        try {
            Remove-Item -LiteralPath $Destination -Force -ErrorAction Stop
        } catch {
            InfoMessage "File $Destination is in use or locked. Renaming it to allow overwrite..."
            $oldPath = "$Destination.old"
            try {
                if (Test-Path -LiteralPath $oldPath) {
                    Remove-Item -LiteralPath $oldPath -Force -ErrorAction SilentlyContinue
                }
                Move-Item -LiteralPath $Destination -Destination $oldPath -Force -ErrorAction Stop
                InfoMessage "Successfully renamed locked file to $oldPath"
            } catch {
                WarnMessage "Failed to rename locked file: $($_.Exception.Message)"
            }
        }
    }
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

    # Handle existing files, resolving file locks if the process is still running
    Prepare-DestinationFile -Destination $Destination

    $attempt = 0
    while ($attempt -lt $MaxRetries) {
        try {
            Invoke-WebRequest -Uri $Url -OutFile $Destination -UseBasicParsing
            SuccessMessage "$Description downloaded successfully"
            return
        } catch {
            $attempt++
            WarnMessage "Download attempt $attempt failed: $($_.Exception.Message)"
            if ($attempt -lt $MaxRetries) {
                WarnMessage "Retrying in 2 seconds..."
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
        [string]$ChecksumFile = $script:ChecksumsPath,
        [string]$ChecksumUrl = $script:ChecksumsURL
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

# Ensure the script is running on Windows
function EnsureWindows {
    if ($PSVersionTable.PSEdition -eq "Core" -and -not $IsWindows) {
        ErrorExit "This script is intended for Windows systems. Please use the appropriate script for your operating system."
    }
}

# Ensure the script is running with administrator privileges
function EnsureAdmin {
    if (-not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
        ErrorExit "This script requires administrative privileges. Please run it as Administrator."
    }
}