!macro AddGhostZipArchiveMenu EXT
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\${EXT}\shell\GhostZip" "MUIVerb" "GhostZip"
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\${EXT}\shell\GhostZip" "Icon" "$INSTDIR\GhostZip.exe"
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\${EXT}\shell\GhostZip" "SubCommands" ""

  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\${EXT}\shell\GhostZip\shell\Analyze" "MUIVerb" "Analyze archive with GhostZip"
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\${EXT}\shell\GhostZip\shell\Analyze\command" "" '"$INSTDIR\GhostZip.exe" --action analyze --archive "%1"'

  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\${EXT}\shell\GhostZip\shell\ExtractHere" "MUIVerb" "Extract here with GhostZip"
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\${EXT}\shell\GhostZip\shell\ExtractHere\command" "" '"$INSTDIR\GhostZip.exe" --action extract-here --archive "%1"'

  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\${EXT}\shell\GhostZip\shell\ExtractToFolder" "MUIVerb" "Extract to archive folder with GhostZip"
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\${EXT}\shell\GhostZip\shell\ExtractToFolder\command" "" '"$INSTDIR\GhostZip.exe" --action extract-to-folder --archive "%1"'
!macroend

!macro RemoveGhostZipArchiveMenu EXT
  DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\${EXT}\shell\GhostZip"
!macroend

!macro NSIS_HOOK_POSTINSTALL
  SetShellVarContext current
  !insertmacro AddGhostZipArchiveMenu ".zip"
  !insertmacro AddGhostZipArchiveMenu ".7z"
  !insertmacro AddGhostZipArchiveMenu ".tar"
  !insertmacro AddGhostZipArchiveMenu ".gz"
  !insertmacro AddGhostZipArchiveMenu ".tgz"
  !insertmacro AddGhostZipArchiveMenu ".bz2"
  !insertmacro AddGhostZipArchiveMenu ".xz"
  !insertmacro AddGhostZipArchiveMenu ".rar"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  SetShellVarContext current
  !insertmacro RemoveGhostZipArchiveMenu ".zip"
  !insertmacro RemoveGhostZipArchiveMenu ".7z"
  !insertmacro RemoveGhostZipArchiveMenu ".tar"
  !insertmacro RemoveGhostZipArchiveMenu ".gz"
  !insertmacro RemoveGhostZipArchiveMenu ".tgz"
  !insertmacro RemoveGhostZipArchiveMenu ".bz2"
  !insertmacro RemoveGhostZipArchiveMenu ".xz"
  !insertmacro RemoveGhostZipArchiveMenu ".rar"
!macroend
