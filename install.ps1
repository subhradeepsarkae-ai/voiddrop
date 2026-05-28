$url = "https://github.com/subhradeepsarkae-ai/voiddrop/releases/latest/download/vb-windows-x64.exe"
$out = Join-Path $env:USERPROFILE "vb.exe"
Write-Host "Downloading vb..." -ForegroundColor Cyan
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -Uri $url -OutFile $out
$path = [Environment]::GetFolderPath("Desktop")
$shortcut = Join-Path $path "vb.exe"
if (!(Test-Path $shortcut)) { New-Item -ItemType SymbolicLink -Path $shortcut -Target $out -Force > $null }
Write-Host "Done! vb.exe is on your Desktop." -ForegroundColor Green
