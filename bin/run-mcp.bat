@echo off
set SRC=%~dp0..\target\debug\mkgp2-mcp.exe
set DST=%TEMP%\mkgp2-mcp-%RANDOM%.exe
copy /Y "%SRC%" "%DST%" >nul 2>&1
if errorlevel 1 (
  echo Failed to copy MCP binary 1>&2
  exit /b 1
)
"%DST%"
del /q "%DST%" 2>nul
