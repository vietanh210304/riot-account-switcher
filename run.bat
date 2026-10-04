@echo off
setlocal
cd /d "%~dp0"
title Riot Account Switcher

set "PYTHONW_EXE=%LOCALAPPDATA%\Python\pythoncore-3.14-64\pythonw.exe"
if not exist "%PYTHONW_EXE%" (
    set "PYTHONW_EXE=%LOCALAPPDATA%\Python\bin\python.exe"
)
if not exist "%PYTHONW_EXE%" (
    set "PYTHONW_EXE=pythonw"
)

start "" "%PYTHONW_EXE%" app.py
exit
