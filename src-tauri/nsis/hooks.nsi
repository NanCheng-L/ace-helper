!macro NSIS_HOOK_PREINSTALL
  ; ========== 清理旧版本 "ACE Helper"（v0.2.0 产品名带空格） ==========
  ; v0.3.0 改名为 "ACE-Helper"，NSIS 视为不同产品，需手动清理旧版
  StrCpy $8 ""  ; 旧版卸载命令
  StrCpy $9 ""  ; 旧版安装目录

  ; 先从当前用户注册表查找
  ReadRegStr $8 HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\ACE Helper" "UninstallString"
  ReadRegStr $9 HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\ACE Helper" "InstallLocation"
  ; 没找到则从所有用户注册表查找
  ${If} $8 == ""
    ReadRegStr $8 HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ACE Helper" "UninstallString"
    ReadRegStr $9 HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ACE Helper" "InstallLocation"
  ${EndIf}

  ${If} $8 != ""
    DetailPrint "检测到旧版本 ACE Helper，正在卸载..."

    ; 先结束旧版本进程（匹配 ace-helper 或 ACE Helper）
    nsExec::ExecToStack 'powershell -NoProfile -Command "Get-Process | Where-Object { $_.ProcessName -match ''ace.helper|ACE.Helper'' } | Stop-Process -Force -ErrorAction SilentlyContinue"'
    Pop $1
    Pop $2
    Sleep 1500

    ; 执行旧版卸载程序（静默模式，等待完成）
    ExecWait '"$8" /S'

    ; 清理旧版开机自启注册表
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "ACE Helper"
    DeleteRegValue HKLM "Software\Microsoft\Windows\CurrentVersion\Run" "ACE Helper"

    ; 清理旧版卸载注册表项（如果卸载程序没有完全清理）
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\ACE Helper"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ACE Helper"

    ; 清理旧版安装目录残留文件
    ${If} $9 != ""
      RMDir /r "$9"
    ${EndIf}
    ; 备用：尝试默认安装路径
    RMDir /r "$LOCALAPPDATA\ACE Helper"

    DetailPrint "旧版本清理完成"
  ${EndIf}

  ; ========== 检测当前版本进程是否在运行 ==========
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
