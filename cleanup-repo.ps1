# Guardian Repository Cleanup Script
# This script removes unnecessary files while keeping essential development files

Write-Host "Starting Guardian repository cleanup..." -ForegroundColor Green

# Remove redundant documentation files
Write-Host "Removing redundant documentation files..." -ForegroundColor Yellow
Remove-Item -Path "DESKTOP_APP_README.md" -ErrorAction SilentlyContinue
Remove-Item -Path "DEVELOPMENT.md" -ErrorAction SilentlyContinue
Remove-Item -Path "IMPLEMENTATION_SUMMARY.md" -ErrorAction SilentlyContinue
Remove-Item -Path "INSTALLER_SETUP.md" -ErrorAction SilentlyContinue
Remove-Item -Path "PRODUCTION_README.md" -ErrorAction SilentlyContinue
Remove-Item -Path "PRODUCTION_SUMMARY.md" -ErrorAction SilentlyContinue
Remove-Item -Path "QUICK_START.md" -ErrorAction SilentlyContinue
Remove-Item -Path "API_SETUP.md" -ErrorAction SilentlyContinue

# Remove build artifacts (will be regenerated)
Write-Host "Removing build artifacts..." -ForegroundColor Yellow
Remove-Item -Path "target" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "gpu-worker/target" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "hostd/target" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "guardian-ui/src-tauri/target" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "guardian-ui/dist" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "guardian-ui/node_modules" -Recurse -ErrorAction SilentlyContinue

# Remove runtime data directories
Write-Host "Removing runtime data directories..." -ForegroundColor Yellow
Remove-Item -Path "data" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "world" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "mods" -Recurse -ErrorAction SilentlyContinue

# Remove debug and output files
Write-Host "Removing debug files..." -ForegroundColor Yellow
Get-ChildItem -Path "gpu-worker" -Name "gpu-error*.txt" | ForEach-Object { Remove-Item -Path "gpu-worker/$_" -ErrorAction SilentlyContinue }
Get-ChildItem -Path "gpu-worker" -Name "gpu-output*.txt" | ForEach-Object { Remove-Item -Path "gpu-worker/$_" -ErrorAction SilentlyContinue }

# Remove unused executables
Write-Host "Removing unused executables..." -ForegroundColor Yellow
Remove-Item -Path "hostd/rustup-init.exe" -ErrorAction SilentlyContinue
Remove-Item -Path "hostd/vs_buildtools.exe" -ErrorAction SilentlyContinue
Remove-Item -Path "guardian-ui/src-tauri/gpu-worker.exe" -ErrorAction SilentlyContinue
Remove-Item -Path "guardian-ui/src-tauri/hostd.exe" -ErrorAction SilentlyContinue

# Remove duplicate config directory
Write-Host "Removing duplicate config directory..." -ForegroundColor Yellow
Remove-Item -Path "config" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "configs/test.yaml" -ErrorAction SilentlyContinue

# Remove launcher scripts (not essential for core functionality)
Write-Host "Removing launcher scripts..." -ForegroundColor Yellow
Remove-Item -Path "Launch-Guardian.bat" -ErrorAction SilentlyContinue
Remove-Item -Path "Setup-Guardian-Shortcuts.bat" -ErrorAction SilentlyContinue
Remove-Item -Path "Update-Shortcut.bat" -ErrorAction SilentlyContinue
Remove-Item -Path "launchers" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "tools" -Recurse -ErrorAction SilentlyContinue

# Remove Docker files (not currently implemented)
Write-Host "Removing Docker files..." -ForegroundColor Yellow
Remove-Item -Path "docker-compose.yml" -ErrorAction SilentlyContinue
Remove-Item -Path "docker" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "gpu-worker/Dockerfile" -ErrorAction SilentlyContinue
Remove-Item -Path "hostd/Dockerfile" -ErrorAction SilentlyContinue

# Remove monitoring files (not currently implemented)
Write-Host "Removing monitoring files..." -ForegroundColor Yellow
Remove-Item -Path "monitoring" -Recurse -ErrorAction SilentlyContinue

# Remove PowerShell scripts (keep only essential ones)
Write-Host "Removing non-essential PowerShell scripts..." -ForegroundColor Yellow
Remove-Item -Path "post-install-setup.ps1" -ErrorAction SilentlyContinue

Write-Host "Cleanup complete!" -ForegroundColor Green
Write-Host "Repository is now clean and ready for GitHub." -ForegroundColor Green
Write-Host ""
Write-Host "Essential files kept:" -ForegroundColor Cyan
Write-Host "  - README.md (main documentation)" -ForegroundColor White
Write-Host "  - LICENSE (license file)" -ForegroundColor White
Write-Host "  - Cargo.toml/Cargo.lock (Rust workspace)" -ForegroundColor White
Write-Host "  - guardian-ui/ (desktop application)" -ForegroundColor White
Write-Host "  - hostd/ (backend server)" -ForegroundColor White
Write-Host "  - gpu-worker/ (GPU worker - in development)" -ForegroundColor White
Write-Host "  - guardian-agent/ (Minecraft agent - in development)" -ForegroundColor White
Write-Host "  - configs/ (configuration templates)" -ForegroundColor White
Write-Host "  - scripts/ (build scripts)" -ForegroundColor White
Write-Host ""
Write-Host "Run 'git add .' and 'git commit' to commit the changes." -ForegroundColor Yellow
