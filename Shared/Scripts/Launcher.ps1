$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Resolve-Path (Join-Path $ScriptDir "..")

function Invoke-Arcadia {
    param(
        [Parameter(Mandatory = $true)]
        [bool]$Release,
        [Parameter(Mandatory = $true)]
        [string]$Feature
    )

    Push-Location $RootDir
    try {
        if ($Release) {
            Write-Host ""
            Write-Host "Running: cargo run -p arcadia --release --features $Feature"
            cargo run -p arcadia --release --features $Feature
        }
        else {
            Write-Host ""
            Write-Host "Running: cargo run -p arcadia --features $Feature"
            cargo run -p arcadia --features $Feature
        }
    }
    finally {
        Pop-Location
    }
}

while ($true) {
    Clear-Host
    Write-Host "=================================="
    Write-Host " Arcadia Launcher"
    Write-Host "=================================="
    Write-Host "Choose option (type two keys, no Enter needed):"
    Write-Host "  1A) Launch GUI Release"
    Write-Host "  1B) Launch GUI Debug"
    Write-Host "  2A) Launch Headless Release"
    Write-Host "  2B) Launch Headless Debug"
    Write-Host "  0X) Exit"
    Write-Host ""
    Write-Host -NoNewline "Enter choice: "

    $first = [System.Console]::ReadKey($true).KeyChar
    $second = [System.Console]::ReadKey($true).KeyChar
    $choice = ("{0}{1}" -f $first, $second).ToUpperInvariant()
    Write-Host $choice

    switch ($choice) {
        "1A" {
            Invoke-Arcadia -Release $true -Feature "gui"
            Read-Host "Press Enter to continue..."
        }
        "1B" {
            Invoke-Arcadia -Release $false -Feature "gui"
            Read-Host "Press Enter to continue..."
        }
        "2A" {
            Invoke-Arcadia -Release $true -Feature "headless"
            Read-Host "Press Enter to continue..."
        }
        "2B" {
            Invoke-Arcadia -Release $false -Feature "headless"
            Read-Host "Press Enter to continue..."
        }
        { $_ -in @("0X", "00", "0A", "0B") } {
            Write-Host "Goodbye."
            exit 0
        }
        default {
            Write-Host ""
            Write-Host "Invalid option. Use 1A, 1B, 2A, 2B, or 0X."
            Read-Host "Press Enter to continue..."
        }
    }
}
