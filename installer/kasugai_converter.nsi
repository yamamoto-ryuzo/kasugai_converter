; Kasugai Converter NSIS installer
; icon.ico は 16x16 から 256x256 までのマルチサイズ ICO を含む
Unicode True
SetCompressor /SOLID lzma
ManifestDPIAware true
RequestExecutionLevel user

!include "MUI2.nsh"

!define APP_NAME "KasugaiConverter"
!define APP_NAME_DISPLAY "Kasugai Converter"
!define APP_VERSION "0.6.0"
!define APP_VERSION_FILE "0.6.0.0"
!define PUBLISHER "Kasugai"

VIProductVersion "${APP_VERSION_FILE}"
VIAddVersionKey "ProductName" "${APP_NAME_DISPLAY}"
VIAddVersionKey "FileVersion" "${APP_VERSION}"
VIAddVersionKey "ProductVersion" "${APP_VERSION}"
VIAddVersionKey "Publisher" "${PUBLISHER}"
VIAddVersionKey "LegalCopyright" "Copyright (c) ${PUBLISHER}"

!define MUI_ICON "icon.ico"
!define MUI_UNICON "icon.ico"
; カスタム画像は使用しないため、既定の MUI 画像を使用

; デフォルトインストール先: ユーザー書き込み可能
InstallDir "C:\kasugai\kasugai_converter"
Name "${APP_NAME_DISPLAY} ${APP_VERSION}"
OutFile "..\download\kasugai_converter_setup.exe"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

!define MUI_FINISHPAGE_RUN "$INSTDIR\kasugai_converter.exe"
!define MUI_FINISHPAGE_RUN_PARAMETERS "--open-browser"
!define MUI_FINISHPAGE_SHOWREADME
!define MUI_FINISHPAGE_SHOWREADME_TEXT "Create desktop shortcut"
!define MUI_FINISHPAGE_SHOWREADME_FUNCTION CreateDesktopShortcut
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "Japanese"

Function .onInit
  ; 既定のインストール先の親ディレクトリを作成
  CreateDirectory "C:\kasugai"
FunctionEnd

Function CreateDesktopShortcut
  CreateShortcut "$DESKTOP\${APP_NAME}.lnk" \
      "$INSTDIR\kasugai_converter.exe" \
      "--open-browser" \
      "$INSTDIR\kasugai_converter.exe" 0 SW_SHOWNORMAL "" "" "$INSTDIR"
FunctionEnd

Section "Install" SecInstall
  SetOutPath "$INSTDIR"

  ; 配布物を含む dist ディレクトリからコピー
  File /r "..\dist\*.*"

  ; スタートメニューショートカット
  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" \
      "$INSTDIR\kasugai_converter.exe" \
      "--open-browser" \
      "$INSTDIR\kasugai_converter.exe" 0 SW_SHOWNORMAL "" "" "$INSTDIR"

  ; レジストリ登録（アンインストール用）
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" \
      "DisplayName" "${APP_NAME_DISPLAY}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" \
      "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" \
      "DisplayIcon" "$INSTDIR\kasugai_converter.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" \
      "Publisher" "${PUBLISHER}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" \
      "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" \
      "InstallLocation" "$INSTDIR"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\uninstall.exe"
  RMDir /r "$INSTDIR"
  Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
  RMDir "$SMPROGRAMS\${APP_NAME}"
  Delete "$DESKTOP\${APP_NAME}.lnk"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
SectionEnd
