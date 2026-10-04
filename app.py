import sys
import threading
import time
import os
import ctypes
from pathlib import Path

# Explicitly register Windows AppUserModelID so Taskbar isolates and shows the custom icon
try:
    myappid = "riot.account.switcher.v2"
    ctypes.windll.shell32.SetCurrentProcessExplicitAppUserModelID(myappid)
except Exception:
    pass

# Ensure directory is on sys.path
if getattr(sys, "frozen", False):
    BUNDLE_DIR = Path(sys._MEIPASS)
    APP_DIR = Path(sys.executable).parent
    ICON_PATH = BUNDLE_DIR / "assets" / "icon.ico"
    if not ICON_PATH.exists():
        ICON_PATH = APP_DIR / "assets" / "icon.ico"
else:
    BUNDLE_DIR = Path(__file__).resolve().parent
    APP_DIR = Path(__file__).resolve().parent
    ICON_PATH = BUNDLE_DIR / "assets" / "icon.ico"

sys.path.insert(0, str(BUNDLE_DIR))
import backend
import webview

def start_backend():
    backend.run_server(48200)

def set_native_window_icon():
    """Sets the Win32 window big/small icon via SendMessage (WM_SETICON) directly to HWND."""
    if not ICON_PATH.exists():
        return
    for _ in range(30):
        time.sleep(0.3)
        hwnd = ctypes.windll.user32.FindWindowW(None, "Riot Account Switcher - LoL & Valorant")
        if hwnd:
            try:
                IMAGE_ICON = 1
                LR_LOADFROMFILE = 0x00000010
                LR_DEFAULTSIZE = 0x00000040
                WM_SETICON = 0x0080
                ICON_SMALL = 0
                ICON_BIG = 1

                # Load big (taskbar / Alt-Tab)
                hicon_big = ctypes.windll.user32.LoadImageW(
                    None, str(ICON_PATH), IMAGE_ICON, 0, 0, LR_LOADFROMFILE | LR_DEFAULTSIZE
                )
                # Load small (title bar)
                hicon_small = ctypes.windll.user32.LoadImageW(
                    None, str(ICON_PATH), IMAGE_ICON, 16, 16, LR_LOADFROMFILE
                )

                if hicon_big:
                    ctypes.windll.user32.SendMessageW(hwnd, WM_SETICON, ICON_BIG, hicon_big)
                if hicon_small:
                    ctypes.windll.user32.SendMessageW(hwnd, WM_SETICON, ICON_SMALL, hicon_small)
                
                # Notify Windows Shell of icon change
                ctypes.windll.shell32.SHChangeNotify(0x08000000, 0x0000, None, None)
                break
            except Exception:
                pass

def main():
    # Start backend in background thread
    t_server = threading.Thread(target=start_backend, daemon=True)
    t_server.start()

    # Start icon enforcement thread
    t_icon = threading.Thread(target=set_native_window_icon, daemon=True)
    t_icon.start()

    # Wait briefly for server ready
    time.sleep(0.4)

    # Create native Windows desktop window
    window = webview.create_window(
        title="Riot Account Switcher - LoL & Valorant",
        url="http://127.0.0.1:48200",
        width=1180,
        height=780,
        min_size=(960, 640),
        resizable=True,
        text_select=False,
        easy_drag=False
    )

    # Pass icon path to webview.start so WinForms/WPF natively loads it
    icon_arg = str(ICON_PATH) if ICON_PATH.exists() else None
    webview.start(icon=icon_arg)

if __name__ == "__main__":
    main()
