$ErrorActionPreference = "Stop"

$script_directory_path = Split-Path -Parent $MyInvocation.MyCommand.Path
$apk_file_path = Join-Path $script_directory_path "squalr.apk"
$worker_binary_file_path = Join-Path $script_directory_path "squalr-cli"
$worker_device_path = "/data/local/tmp/squalr-cli"

function Invoke-AdbCommand {
    param (
        [Parameter(Mandatory = $true)]
        [string[]]$CommandSegments
    )

    Write-Host "`n> adb $($CommandSegments -join ' ')"
    & adb @CommandSegments
    if ($LASTEXITCODE -ne 0) {
        throw "adb command failed with exit code $LASTEXITCODE."
    }
}

if (-not (Get-Command adb -ErrorAction SilentlyContinue)) {
    throw "Missing required command: adb"
}

if (-not (Test-Path -LiteralPath $apk_file_path)) {
    throw "Missing APK artifact: $apk_file_path"
}

if (-not (Test-Path -LiteralPath $worker_binary_file_path)) {
    throw "Missing worker artifact: $worker_binary_file_path"
}

Write-Host "Installing APK..."
Invoke-AdbCommand -CommandSegments @("install", "-r", $apk_file_path)

Write-Host "Pushing worker binary..."
Invoke-AdbCommand -CommandSegments @("push", $worker_binary_file_path, $worker_device_path)

$su_invocation_segments_list = @(
    @("shell", "su", "-c", "chmod +x $worker_device_path"),
    @("shell", "su", "0", "sh", "-c", "chmod +x $worker_device_path"),
    @("shell", "su", "root", "sh", "-c", "chmod +x $worker_device_path")
)

$was_chmod_successful = $false
foreach ($su_invocation_segments in $su_invocation_segments_list) {
    Write-Host "`n> adb $($su_invocation_segments -join ' ')"
    & adb @su_invocation_segments
    if ($LASTEXITCODE -eq 0) {
        $was_chmod_successful = $true
        break
    }
}

if (-not $was_chmod_successful) {
    throw "Failed to chmod worker binary with supported su invocations."
}

Write-Host "Install complete."
