!macro NSIS_HOOK_PREINSTALL
  ; 检测旧版本进程是否仍在运行
  retry:
  nsExec::ExecToStack 'powershell -NoProfile -Command "if(Get-Process -Name ${MAINBINARYNAME} -ErrorAction SilentlyContinue){exit 1}else{exit 0}"'
  Pop $0
  Pop $1
  ${If} $0 == 1
    MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "${PRODUCTNAME} 正在运行，请先关闭后再继续安装。$\n$\n关闭后点击「重试」继续，或点击「取消」退出安装。" IDRETRY retry
    Quit
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}" '"$INSTDIR\${MAINBINARYNAME}.exe" --hidden'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}"
!macroend
