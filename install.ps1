$url = "https://github.com/subhradeepsarkae-ai/voiddrop/releases/latest/download/vb.exe"
$dir = "$env:USERPROFILE\.voiddrop"
$out = "$dir\vb.exe"
if (!(Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
Write-Host "Downloading vb..." -ForegroundColor Cyan
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -Uri $url -OutFile $out
$path = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($path -notlike "*$dir*") {
  [Environment]::SetEnvironmentVariable("PATH", "$path;$dir", "User")
  $env:PATH += ";$dir"
}
Write-Host "Done! vb is ready to use anywhere." -ForegroundColor Green
Write-Host "Just type:  vb send <FILE> --fast" -ForegroundColor Cyan
