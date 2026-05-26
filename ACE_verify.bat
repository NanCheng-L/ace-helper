@echo off
chcp 437 >nul 2>&1
setlocal enabledelayedexpansion

echo ==============================================
echo ACE Process Config - Verification Script
echo ==============================================

REM 获取 CPU 核心总数，计算最后一个核心索引
for /f %%c in ('powershell -NoProfile -Command "[System.Environment]::ProcessorCount"') do set "cpu_total=%%c"
set /a "last_core=%cpu_total%-1"
set /a "aff_mask=1<<%last_core%"

echo CPU Cores: %cpu_total% (targeting last core: %last_core%)
echo.

set "p1=SGuardSvc64.exe"
set "p2=SGuard64.exe"

for %%p in ("%p1%" "%p2%") do (
    set "found=0"
    for /f "tokens=2 delims=," %%a in ('tasklist /fi "imagename eq %%~p" /fo csv /nh 2^>nul') do (
        set "pid=%%~a"
        set "found=1"
    )
    if "!found!"=="1" (
        echo [%%~p] PID: !pid!
        powershell -NoProfile -Command ^
            "$p=Get-Process -Id !pid! -ErrorAction Stop;" ^
            "$p.PriorityClass='Idle';" ^
            "$p.ProcessorAffinity=%aff_mask%;" ^
            "Write-Host '  Priority: Idle';" ^
            "Write-Host ('  Affinity: Core %last_core% (mask: ' + $p.ProcessorAffinity + ')');" ^
            "Write-Host '  [OK] Priority && Affinity configured'"
        
        REM 设置磁盘 I/O 优先级为 Very Low (1)
        REM 使用 Windows API 通过 PowerShell
        powershell -NoProfile -Command ^
            "Add-Type @\"" ^
            "using System;" ^
            "using System.Runtime.InteropServices;" ^
            "public class IoPriority {" ^
            "    [DllImport(\"kernel32.dll\")] public static extern IntPtr OpenProcess(uint access, bool inherit, int pid);" ^
            "    [DllImport(\"kernel32.dll\")] public static extern bool CloseHandle(IntPtr handle);" ^
            "    [DllImport(\"ntdll.dll\")] public static extern int NtSetInformationProcess(IntPtr h, int cls, ref int info, int size);" ^
            "    public const int PROCESS_SET_INFORMATION = 0x0200;" ^
            "    public const int PROCESS_QUERY_INFORMATION = 0x0400;" ^
            "    public const int ProcessIoPriority = 33;" ^
            "    public const int IOPriority_VeryLow = 1;" ^
            "}" ^
            "\"@;" ^
            "$h=[IoPriority]::OpenProcess(0x0600,$false,!pid!);" ^
            "if($h -ne 0){" ^
            "    $prio=1;" ^
            "    $r=[IoPriority]::NtSetInformationProcess($h,33,[ref]$prio,4);" ^
            "    [IoPriority]::CloseHandle($h);" ^
            "    if($r -eq 0){Write-Host '  I/O Priority: Very Low [OK]'}else{Write-Host ('  I/O Priority: Failed (0x' + $r.ToString('X8') + ')')}" ^
            "}else{Write-Host '  I/O Priority: Failed to open process'}"
        
        REM 注意：效率模式仅支持 Windows 11+，请使用 ACE 小助手主程序设置
    ) else (
        echo [%%~p] Not running
    )
    echo.
)

echo ==============================================
echo Done. Press any key to exit...
pause >nul
