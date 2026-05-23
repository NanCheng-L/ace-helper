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
            "if($p.PriorityClass -eq 'Idle'){Write-Host '  Priority: Idle [OK]'}else{Write-Host '  Priority: FAILED'};" ^
            "if($p.ProcessorAffinity -eq %aff_mask%){Write-Host '  Affinity: Core %last_core% [OK]'}else{Write-Host ('  Affinity: '+$p.ProcessorAffinity+' [PARTIAL]')}"
    ) else (
        echo [%%~p] Not running
    )
    echo.
)

echo Done. Press any key to exit...
pause >nul