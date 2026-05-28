$dir = "$env:USERPROFILE\.voiddrop"
$old = "$env:USERPROFILE\vb.exe"
Write-Host "Uninstalling vb..." -ForegroundColor Yellow

# Remove vb.exe from PATH folder
if (Test-Path "$dir\vb.exe") { Remove-Item "$dir\vb.exe" -Force; Write-Host "  ✓ Removed vb.exe" -ForegroundColor Green }

# Remove the .voiddrop folder
if (Test-Path $dir) { Remove-Item $dir -Force; Write-Host "  ✓ Removed .voiddrop folder" -ForegroundColor Green }

# Remove old leftover from earlier install
if (Test-Path $old) { Remove-Item $old -Force; Write-Host "  ✓ Removed old vb.exe" -ForegroundColor Green }

# Remove PATH entry
$path = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($path -like "*\.voiddrop*") {
  $new = ($path -split ';' | Where-Object { $_ -notlike '*\.voiddrop*' }) -join ';'
  [Environment]::SetEnvironmentVariable("PATH", $new, "User")
  Write-Host "  ✓ Removed from PATH" -ForegroundColor Green
}

Write-Host ""
Write-Host "vb has been uninstalled." -ForegroundColor Green
Write-Host "To reinstall: iwr -useb https://raw.githubusercontent.com/subhradeepsarkae-ai/voiddrop/master/install.ps1 | iex" -ForegroundColor Cyan
