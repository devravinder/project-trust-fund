@echo off
REM Initializes the MSVC build environment (needed to compile libsql's bundled
REM sqlite3.c) and then runs whatever command is passed as arguments.
REM
REM Usage:  scripts\with-msvc.bat pnpm tauri dev
REM
REM It locates Visual Studio / Build Tools via vswhere, so it keeps working
REM even if the install path or version changes.

setlocal

set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "%VSWHERE%" (
  echo [with-msvc] vswhere.exe not found. Install Visual Studio Build Tools with the C++ workload.
  exit /b 1
)

for /f "usebackq tokens=*" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "VSPATH=%%i"

if "%VSPATH%"=="" (
  echo [with-msvc] No VS install with the C++ toolset found. Install the "Desktop development with C++" workload.
  exit /b 1
)

set "VCVARS=%VSPATH%\VC\Auxiliary\Build\vcvars64.bat"
if not exist "%VCVARS%" (
  echo [with-msvc] vcvars64.bat not found at "%VCVARS%".
  exit /b 1
)

call "%VCVARS%" >nul
if errorlevel 1 (
  echo [with-msvc] Failed to initialize MSVC environment.
  exit /b 1
)

REM Run the passed-through command.
%*
