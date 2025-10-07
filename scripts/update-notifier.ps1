param(
    [Parameter(Mandatory = $true)]
    [string]$UpdateScriptPath,
    [string]$LogFile = "${env:ProgramFiles(x86)}\ossec-agent\active-response\active-responses.log",
    [string]$ServiceName
)

$ErrorActionPreference = "Stop"

function Log {
    param (
        [string]$Level,
        [string]$Message
    )
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    "$timestamp [$Level] $Message" | Out-File -FilePath $LogFile -Append -Encoding UTF8
}

function Info { param([string]$m) Log "INFO" $m }
function ErrorLog { param([string]$m) Log "ERROR" $m }

function Get-ActiveUserAndSessionId {
    # Try to get the active console session id and associated user
    $sessionId = (Get-CimInstance -ClassName Win32_LogonSession -Filter "LogonType = 2" | Sort-Object StartTime -Descending | Select-Object -First 1).LogonId
    if (-not $sessionId) {
        return $null
    }
    $link = Get-CimInstance -ClassName Win32_LoggedOnUser | Where-Object { $_.Dependent -match "LogonId=\"$sessionId\"" } | Select-Object -First 1
    if (-not $link) {
        return $null
    }
    $userPath = $link.Antecedent
    $m = [regex]::Match($userPath, 'Domain=\"([^\"]+)\",Name=\"([^\"]+)\"')
    if ($m.Success) {
        return @{ User = "$($m.Groups[1].Value)\\$($m.Groups[2].Value)"; SessionId = $sessionId }
    }
    return $null
}

function Show-InteractivePopupViaScheduledTask {
    param(
        [Parameter(Mandatory=$true)][string]$Title,
        [Parameter(Mandatory=$true)][string]$Message
    )
    try {
        $active = Get-ActiveUserAndSessionId
        if (-not $active) {
            Info "No interactive user detected; skipping popup."
            return
        }

        $taskName = "WazuhUpdateNotify_" + ([System.Guid]::NewGuid().ToString())
        $script = "$env:WINDIR\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"
        $popupCmd = "-NoProfile -WindowStyle Hidden -Command \"Add-Type -AssemblyName PresentationFramework; [System.Windows.MessageBox]::Show('$($Message.Replace("'","''"))','$($Title.Replace("'","''"))')\""

        $action = New-ScheduledTaskAction -Execute $script -Argument $popupCmd
        $trigger = New-ScheduledTaskTrigger -Once -At (Get-Date).AddSeconds(5)
        $principal = New-ScheduledTaskPrincipal -UserId $active.User -LogonType Interactive -RunLevel Highest
        $task = New-ScheduledTask -Action $action -Trigger $trigger -Principal $principal
        Register-ScheduledTask -TaskName $taskName -InputObject $task | Out-Null
        Start-ScheduledTask -TaskName $taskName | Out-Null
        Start-Sleep -Seconds 10
        Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue | Out-Null
        Info "Scheduled task '$taskName' created to display popup for $($active.User)."
    } catch {
        ErrorLog ("Failed to display interactive popup: {0}" -f $_.Exception.Message)
    }
}

if (-not (Test-Path -Path $UpdateScriptPath)) {
    ErrorLog "Update script not found at '$UpdateScriptPath'. Exiting notifier."
    exit 1
}

try {
    Info "Starting update-notifier to monitor '$UpdateScriptPath'..."

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = "$env:WINDIR\System32\WindowsPowerShell\v1.0\powershell.exe"
    $psi.Arguments = "-ExecutionPolicy Bypass -File `"$UpdateScriptPath`""
    $psi.WindowStyle = [System.Diagnostics.ProcessWindowStyle]::Hidden
    $psi.CreateNoWindow = $true
    $psi.UseShellExecute = $false

    $proc = New-Object System.Diagnostics.Process
    $proc.StartInfo = $psi
    $null = $proc.Start()

    $proc.EnableRaisingEvents = $true

    $sourceId = "WazuhUpdateProcessExited"
    Register-ObjectEvent -InputObject $proc -EventName Exited -SourceIdentifier $sourceId -Action {
        try {
            $code = $Event.Sender.ExitCode
            if ($code -eq 0) {
                Log "INFO" "Wazuh agent update completed successfully. ExitCode=$code"
                Show-InteractivePopupViaScheduledTask -Title "Wazuh Agent Update" -Message "Update completed successfully. Please reboot to finish."
            } else {
                Log "ERROR" "Wazuh agent update failed. ExitCode=$code"
                Show-InteractivePopupViaScheduledTask -Title "Wazuh Agent Update" -Message "Update failed. Please contact IT or retry later."
            }
            if ($using:ServiceName) {
                try {
                    sc.exe delete $using:ServiceName | Out-Null
                    Log "INFO" ("Deleted temporary service '{0}'" -f $using:ServiceName)
                } catch {
                    Log "ERROR" ("Failed to delete temporary service '{0}': {1}" -f $using:ServiceName, $_.Exception.Message)
                }
            }
        } catch {
            Log "ERROR" ("Notifier failed while processing exit event: {0}" -f $_.Exception.Message)
        } finally {
            try { Unregister-Event -SourceIdentifier $using:sourceId -ErrorAction SilentlyContinue } catch { }
            try { Remove-Job -Name $using:sourceId -ErrorAction SilentlyContinue } catch { }
            # Exit the notifier once we've logged the outcome
            Stop-Process -Id $PID -Force
        }
    } | Out-Null

    # Block here until the process exits, to ensure the action runs even if events are delayed
    $proc.WaitForExit()

    # Safety: if for some reason the event didn't trigger, log based on observed ExitCode
    $exit = $proc.ExitCode
    if ($exit -eq 0) {
        Info "Wazuh agent update completed successfully (wait). ExitCode=$exit"
    } else {
        ErrorLog "Wazuh agent update failed (wait). ExitCode=$exit"
    }
} catch {
    ErrorLog ("Notifier encountered an error: {0}" -f $_.Exception.Message)
    exit 1
}

exit 0


