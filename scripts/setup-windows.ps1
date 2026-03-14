# NetPulse — Windows Setup Script
# Run this in PowerShell as Administrator

Write-Host "╔════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║       NetPulse — Windows Setup Script          ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Check if running as Administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "⚠️  This script must be run as Administrator." -ForegroundColor Yellow
    Write-Host "   Right-click PowerShell and select 'Run as Administrator'" -ForegroundColor Yellow
    exit 1
}

# Check for Npcap
$npcapInstalled = Test-Path "C:\Windows\System32\Npcap"
if (-not $npcapInstalled) {
    $npcapInstalled = Get-ItemProperty "HKLM:\SOFTWARE\Npcap" -ErrorAction SilentlyContinue
}

if (-not $npcapInstalled) {
    Write-Host "📦 Npcap is not installed." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "   NetPulse requires Npcap for packet capture." -ForegroundColor White
    Write-Host ""
    Write-Host "   Please download and install Npcap from:" -ForegroundColor White
    Write-Host "   https://npcap.com/#download" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "   During installation, make sure to check:" -ForegroundColor White
    Write-Host "   ✅ Install Npcap in WinPcap API-compatible Mode" -ForegroundColor Green
    Write-Host "   ✅ Support raw 802.11 traffic" -ForegroundColor Green
    Write-Host ""
    
    $response = Read-Host "Would you like to open the download page now? (Y/N)"
    if ($response -eq "Y" -or $response -eq "y") {
        Start-Process "https://npcap.com/#download"
    }
    
    Write-Host ""
    Write-Host "After installing Npcap, run this script again." -ForegroundColor Yellow
    exit 0
} else {
    Write-Host "✅ Npcap is installed." -ForegroundColor Green
}

# Check for Rust
$rustInstalled = Get-Command rustc -ErrorAction SilentlyContinue
if (-not $rustInstalled) {
    Write-Host ""
    Write-Host "🦀 Rust is not installed." -ForegroundColor Yellow
    Write-Host "   Installing via rustup..." -ForegroundColor White
    
    $rustupUrl = "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
    $rustupPath = "$env:TEMP\rustup-init.exe"
    
    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
    Start-Process -FilePath $rustupPath -ArgumentList "-y" -Wait
    
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    
    Write-Host "   ✅ Rust installed." -ForegroundColor Green
} else {
    Write-Host "🦀 Rust: $(rustc --version)" -ForegroundColor Green
}

# Check for Node.js
$nodeInstalled = Get-Command node -ErrorAction SilentlyContinue
if (-not $nodeInstalled) {
    Write-Host ""
    Write-Host "📦 Node.js is not installed." -ForegroundColor Yellow
    Write-Host "   Please install Node.js 18+ from: https://nodejs.org/" -ForegroundColor White
    exit 1
} else {
    Write-Host "📦 Node.js: $(node --version)" -ForegroundColor Green
}

Write-Host ""
Write-Host "📥 Installing npm dependencies..." -ForegroundColor White
npm install

Write-Host ""
Write-Host "🔨 Building NetPulse..." -ForegroundColor White
npm run tauri build

Write-Host ""
Write-Host "╔════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║              Setup Complete! 🎉               ║" -ForegroundColor Green
Write-Host "╠════════════════════════════════════════════════╣" -ForegroundColor Green
Write-Host "║                                                ║" -ForegroundColor Green
Write-Host "║  To run in dev mode:                           ║" -ForegroundColor Green
Write-Host "║    npm run tauri dev                           ║" -ForegroundColor Green
Write-Host "║                                                ║" -ForegroundColor Green
Write-Host "║  The installer is at:                          ║" -ForegroundColor Green
Write-Host "║    src-tauri\target\release\bundle\             ║" -ForegroundColor Green
Write-Host "║                                                ║" -ForegroundColor Green
Write-Host "║  ⚠️  Remember to download GeoLite2-City.mmdb    ║" -ForegroundColor Green
Write-Host "║  and place it in %APPDATA%\netpulse\           ║" -ForegroundColor Green
Write-Host "║                                                ║" -ForegroundColor Green
Write-Host "╚════════════════════════════════════════════════╝" -ForegroundColor Green
