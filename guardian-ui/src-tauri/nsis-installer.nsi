; Guardian Custom NSIS Installer Script
!define APPNAME "Guardian - Minecraft Server Manager"
!define COMPANYNAME "Guardian Team"
!define DESCRIPTION "Professional Minecraft Server Management"
!define VERSIONMAJOR 1
!define VERSIONMINOR 0
!define VERSIONBUILD 0
!define HELPURL "https://github.com/guardian-team/guardian"
!define UPDATEURL "https://github.com/guardian-team/guardian/releases"
!define ABOUTURL "https://github.com/guardian-team/guardian"
!define INSTALLSIZE 50000

RequestExecutionLevel admin
InstallDir "$PROGRAMFILES64\${APPNAME}"
Name "${APPNAME}"
outFile "Guardian_Setup.exe"

!include LogicLib.nsh
!include MUI2.nsh

; Interface Settings
!define MUI_ABORTWARNING
!define MUI_ICON "icons\icon.ico"
!define MUI_UNICON "icons\icon.ico"

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Languages
!insertmacro MUI_LANGUAGE "English"

!macro VerifyUserIsAdmin
UserInfo::GetAccountType
pop $0
${If} $0 != "admin"
    messageBox mb_iconstop "Administrator rights required!"
    setErrorLevel 740
    quit
${EndIf}
!macroend

function .onInit
    setShellVarContext all
    !insertmacro VerifyUserIsAdmin
functionEnd

section "install"
    setOutPath $INSTDIR
    
    ; Copy the main executable
    file "guardian.exe"
    
    ; Copy backend services
    file "hostd.exe"
    file "gpu-worker.exe"
    
    ; Copy configuration files
    file /r "configs"
    
    ; Copy launcher scripts
    file "start-guardian-with-backend.bat"
    file "start-guardian-with-backend.ps1"
    file "create-desktop-shortcut.ps1"
    file "update-desktop-shortcut.ps1"
    
    ; Create data directory
    createDirectory "$INSTDIR\data"
    
    ; Create desktop shortcut to the launcher batch file
    createShortCut "$DESKTOP\${APPNAME}.lnk" "$INSTDIR\start-guardian-with-backend.bat" "" "$INSTDIR\guardian.exe" 0
    
    ; Create start menu shortcuts
    createDirectory "$SMPROGRAMS\${APPNAME}"
    createShortCut "$SMPROGRAMS\${APPNAME}\${APPNAME}.lnk" "$INSTDIR\start-guardian-with-backend.bat" "" "$INSTDIR\guardian.exe" 0
    createShortCut "$SMPROGRAMS\${APPNAME}\Guardian (PowerShell).lnk" "powershell.exe" "-ExecutionPolicy Bypass -File `"$INSTDIR\start-guardian-with-backend.ps1`"" "$INSTDIR\guardian.exe" 0
    createShortCut "$SMPROGRAMS\${APPNAME}\Create Desktop Shortcut.lnk" "powershell.exe" "-ExecutionPolicy Bypass -File `"$INSTDIR\create-desktop-shortcut.ps1`"" "$INSTDIR\guardian.exe" 0
    createShortCut "$SMPROGRAMS\${APPNAME}\Uninstall.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\uninstall.exe" 0
    
    ; Write uninstaller
    writeUninstaller "$INSTDIR\uninstall.exe"
    
    ; Write registry keys for uninstaller
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayName" "${APPNAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayIcon" "$INSTDIR\guardian.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "Publisher" "${COMPANYNAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "HelpLink" "${HELPURL}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "URLUpdateInfo" "${UPDATEURL}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "URLInfoAbout" "${ABOUTURL}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayVersion" "${VERSIONMAJOR}.${VERSIONMINOR}.${VERSIONBUILD}"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "VersionMajor" ${VERSIONMAJOR}
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "VersionMinor" ${VERSIONMINOR}
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "NoRepair" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "EstimatedSize" ${INSTALLSIZE}
sectionEnd

section "uninstall"
    ; Remove files
    delete "$INSTDIR\guardian.exe"
    delete "$INSTDIR\hostd.exe"
    delete "$INSTDIR\gpu-worker.exe"
    delete "$INSTDIR\start-guardian-with-backend.bat"
    delete "$INSTDIR\start-guardian-with-backend.ps1"
    delete "$INSTDIR\create-desktop-shortcut.ps1"
    delete "$INSTDIR\update-desktop-shortcut.ps1"
    delete "$INSTDIR\uninstall.exe"
    
    ; Remove directories
    rmDir /r "$INSTDIR\configs"
    rmDir /r "$INSTDIR\data"
    rmDir "$INSTDIR"
    
    ; Remove shortcuts
    delete "$DESKTOP\${APPNAME}.lnk"
    delete "$SMPROGRAMS\${APPNAME}\${APPNAME}.lnk"
    delete "$SMPROGRAMS\${APPNAME}\Guardian (PowerShell).lnk"
    delete "$SMPROGRAMS\${APPNAME}\Create Desktop Shortcut.lnk"
    delete "$SMPROGRAMS\${APPNAME}\Uninstall.lnk"
    rmDir "$SMPROGRAMS\${APPNAME}"
    
    ; Remove registry keys
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}"
sectionEnd
