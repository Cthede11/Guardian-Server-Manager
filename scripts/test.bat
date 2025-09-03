@echo off
REM Guardian Platform Test Script for Windows
REM This script helps you test the Guardian Platform

setlocal enabledelayedexpansion

REM Colors for output (Windows doesn't support colors in batch, but we can use echo)
set "INFO=[INFO]"
set "SUCCESS=[SUCCESS]"
set "WARNING=[WARNING]"
set "ERROR=[ERROR]"

REM Function to print status
:print_status
echo %INFO% %~1
goto :eof

:print_success
echo %SUCCESS% %~1
goto :eof

:print_warning
echo %WARNING% %~1
goto :eof

:print_error
echo %ERROR% %~1
goto :eof

REM Check if Docker is running
:check_docker
docker info >nul 2>&1
if %errorlevel% neq 0 (
    call :print_error "Docker is not running. Please start Docker Desktop and try again."
    exit /b 1
)
call :print_success "Docker is running"
goto :eof

REM Check if Docker Compose is available
:check_docker_compose
docker-compose --version >nul 2>&1
if %errorlevel% neq 0 (
    call :print_error "Docker Compose is not installed. Please install Docker Compose and try again."
    exit /b 1
)
call :print_success "Docker Compose is available"
goto :eof

REM Check if ports are available
:check_ports
netstat -an | findstr :8080 >nul
if %errorlevel% equ 0 (
    call :print_warning "Port 8080 is already in use. The web interface might not start properly."
) else (
    call :print_success "Port 8080 is available"
)

netstat -an | findstr :25565 >nul
if %errorlevel% equ 0 (
    call :print_warning "Port 25565 is already in use. Minecraft servers might not start properly."
) else (
    call :print_success "Port 25565 is available"
)
goto :eof

REM Start the platform
:start_platform
call :print_status "Starting Guardian Platform..."
if not exist "docker-compose.yml" (
    call :print_error "docker-compose.yml not found. Please run from the project root directory."
    exit /b 1
)

docker-compose up -d
if %errorlevel% neq 0 (
    call :print_error "Failed to start the platform"
    exit /b 1
)

call :print_success "Platform started successfully!"
goto :eof

REM Wait for service to be ready
:wait_for_service
call :print_status "Waiting for web interface to be ready..."
set /a attempts=0
:wait_loop
set /a attempts+=1
if %attempts% gtr 30 (
    call :print_error "Web interface failed to start within expected time"
    exit /b 1
)

curl -s http://localhost:8080 >nul 2>&1
if %errorlevel% equ 0 (
    call :print_success "Web interface is ready!"
    goto :eof
)

echo.
timeout /t 2 /nobreak >nul
goto wait_loop

REM Test web interface
:test_web_interface
call :print_status "Testing web interface..."

set "endpoints=http://localhost:8080 http://localhost:8080/server_management.html http://localhost:8080/performance.html http://localhost:8080/backup.html http://localhost:8080/deployment.html http://localhost:8080/plugins.html http://localhost:8080/users.html http://localhost:8080/settings.html"

for %%e in (%endpoints%) do (
    curl -s "%%e" >nul 2>&1
    if !errorlevel! equ 0 (
        call :print_success "✓ %%e is accessible"
    ) else (
        call :print_error "✗ %%e is not accessible"
    )
)
goto :eof

REM Show platform status
:show_status
call :print_status "Platform Status:"

docker-compose ps
if %errorlevel% neq 0 (
    call :print_warning "No Docker containers are running"
    goto :eof
)

curl -s http://localhost:8080 >nul 2>&1
if %errorlevel% equ 0 (
    call :print_success "Web interface is accessible at http://localhost:8080"
) else (
    call :print_warning "Web interface is not accessible"
)
goto :eof

REM Stop the platform
:stop_platform
call :print_status "Stopping Guardian Platform..."
docker-compose down
if %errorlevel% neq 0 (
    call :print_error "Failed to stop the platform"
    exit /b 1
)
call :print_success "Platform stopped successfully!"
goto :eof

REM Show logs
:show_logs
call :print_status "Showing platform logs..."
docker-compose logs --tail=50
goto :eof

REM Clean up
:cleanup
call :print_status "Cleaning up..."
docker-compose down -v
docker system prune -f
call :print_success "Cleanup completed!"
goto :eof

REM Show help
:show_help
echo Guardian Platform Test Script for Windows
echo.
echo Usage: %0 [COMMAND]
echo.
echo Commands:
echo   start       Start the Guardian Platform
echo   stop        Stop the Guardian Platform
echo   test        Run all tests
echo   web         Test web interface only
echo   status      Show platform status
echo   logs        Show platform logs
echo   cleanup     Stop platform and clean up
echo   help        Show this help message
echo.
echo Examples:
echo   %0 start    # Start the platform
echo   %0 test     # Run all tests
echo   %0 status   # Check if platform is running
goto :eof

REM Main script logic
if "%1"=="" goto show_help
if "%1"=="help" goto show_help
if "%1"=="start" goto start_platform
if "%1"=="stop" goto stop_platform
if "%1"=="test" goto run_all_tests
if "%1"=="web" goto test_web_interface
if "%1"=="status" goto show_status
if "%1"=="logs" goto show_logs
if "%1"=="cleanup" goto cleanup

REM Run all tests
:run_all_tests
call :check_docker
call :check_docker_compose
call :check_ports
call :start_platform
call :wait_for_service
call :test_web_interface
call :show_status
goto :eof

REM If no valid command, show help
goto show_help
