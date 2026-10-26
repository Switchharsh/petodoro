# One-line installer for Windows (PowerShell):
#   irm https://raw.githubusercontent.com/Switchharsh/petodoro/HEAD/get.ps1 | iex
# Downloads the latest release, installs it for the current user (Start menu
# + start on login) and launches your pet.
$ErrorActionPreference = 'Stop'
$url = 'https://github.com/Switchharsh/petodoro/releases/latest/download/petodoro-windows.exe'
$exe = Join-Path $env:TEMP 'petodoro-setup.exe'
Write-Host '==> Downloading Petodoro...'
Invoke-WebRequest -Uri $url -OutFile $exe -UseBasicParsing
& $exe --install
Write-Host '==> Done! Your pet is on the screen and in the Start menu.'
