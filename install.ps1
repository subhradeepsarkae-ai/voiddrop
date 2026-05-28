$url = "https://github.com/subhradeepsarkae-ai/voiddrop/releases/latest/download/vb.exe"
$out = Join-Path $env:USERPROFILE "vb.exe"
Write-Host "Downloading vb..." -ForegroundColor Cyan
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -Uri $url -OutFile $out
Write-Host "Done! vb.exe is at $out" -ForegroundColor Green
Write-Host "Usage:  `"$out`" send <FILE> --fast" -ForegroundColor Yellow
