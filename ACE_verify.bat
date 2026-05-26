@echo off
chcp 437 >nul 2>&1
setlocal enabledelayedexpansion

echo ==============================================
echo ACE Process Config - Verification Script
echo ==============================================

for /f %%c in ('powershell -NoProfile -Command "[System.Environment]::ProcessorCount"') do set "cpu_total=%%c"
echo CPU Cores: %cpu_total%
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
            "$p.ProcessorAffinity=[IntPtr]([int64]1 -shl ([System.Environment]::ProcessorCount-1));" ^
            "Add-Type -TypeDefinition 'using System.Runtime.InteropServices; using System; public class ProcessIO { [DllImport(\"kernel32.dll\")] public static extern IntPtr OpenProcess(uint dwDesiredAccess, bool bInheritHandle, int dwProcessId); [DllImport(\"kernel32.dll\")] public static extern bool SetPriorityClass(IntPtr hProcess, uint dwPriorityClass); [DllImport(\"ntdll.dll\")] public static extern int NtSetInformationProcess(IntPtr hProcess, int processInformationClass, ref int processInformation, int processInformationLength); }';" ^
            "$h=[ProcessIO]::OpenProcess(0x1F0FFF,$false,!pid!);" ^
            "[ProcessIO]::NtSetInformationProcess($h,33,[ref]0,4);" ^
            "Write-Host ('  Priority: ' + $p.PriorityClass);" ^
            "Write-Host ('  Affinity: ' + $p.ProcessorAffinity);" ^
            "Write-Host '  I/O Priority: Very Low'"
    ) else (
        echo [%%~p] Not running
    )
    echo.
)

echo Done. Press any key to exit...
pause >nul