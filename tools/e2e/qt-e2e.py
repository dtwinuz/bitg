# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

"""bitg IME'ning Qt (Telegram Desktop bilan bir xil yo'l: IMM32/CUAS, matn TSF'ga berilmaydi) sinovi.

Tugmalar SendInput bilan bosiladi, natija QTextEdit'dan o'qiladi. FAQAT CI VM'da (GitHub Actions):
lokal kompyuterda SendInput tugmalari fokusdagi har qanday oynaga ketadi.

    pip install PySide6-Essentials
    python tools/e2e/qt-e2e.py --out result-qt.txt
"""

import argparse
import ctypes
import os
import sys
from ctypes import wintypes

if os.environ.get("GITHUB_ACTIONS") != "true":
    sys.exit("Faqat CI VM uchun (SendInput). GITHUB_ACTIONS=true emas.")

from PySide6.QtCore import QTimer  # noqa: E402
from PySide6.QtGui import QFont  # noqa: E402
from PySide6.QtWidgets import QApplication, QLineEdit, QTextEdit  # noqa: E402

user32 = ctypes.WinDLL("user32", use_last_error=True)
ole32 = ctypes.WinDLL("ole32")


class KEYBDINPUT(ctypes.Structure):
    _fields_ = [
        ("wVk", wintypes.WORD),
        ("wScan", wintypes.WORD),
        ("dwFlags", wintypes.DWORD),
        ("time", wintypes.DWORD),
        ("dwExtraInfo", ctypes.c_size_t),
    ]


class MOUSEINPUT(ctypes.Structure):
    _fields_ = [
        ("dx", wintypes.LONG),
        ("dy", wintypes.LONG),
        ("mouseData", wintypes.DWORD),
        ("dwFlags", wintypes.DWORD),
        ("time", wintypes.DWORD),
        ("dwExtraInfo", ctypes.c_size_t),
    ]


class _U(ctypes.Union):
    _fields_ = [("ki", KEYBDINPUT), ("mi", MOUSEINPUT)]


class INPUT(ctypes.Structure):
    _fields_ = [("type", wintypes.DWORD), ("u", _U)]


class GUID(ctypes.Structure):
    _fields_ = [
        ("d1", wintypes.DWORD),
        ("d2", wintypes.WORD),
        ("d3", wintypes.WORD),
        ("d4", ctypes.c_ubyte * 8),
    ]

    @classmethod
    def parse(cls, s):
        h = s.replace("-", "")
        return cls(int(h[0:8], 16), int(h[8:12], 16), int(h[12:16], 16),
                   (ctypes.c_ubyte * 8)(*bytes.fromhex(h[16:32])))


user32.SendInput.argtypes = [wintypes.UINT, ctypes.POINTER(INPUT), ctypes.c_int]
user32.SetForegroundWindow.argtypes = [wintypes.HWND]
user32.GetForegroundWindow.restype = wintypes.HWND

KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE = 0x1, 0x2, 0x8


def scan(code, up=False, ext=False):
    i = INPUT(type=1)
    i.u.ki.wScan = code
    i.u.ki.dwFlags = KEYEVENTF_SCANCODE | (KEYEVENTF_KEYUP if up else 0) | (KEYEVENTF_EXTENDEDKEY if ext else 0)
    user32.SendInput(1, ctypes.byref(i), ctypes.sizeof(INPUT))


def activate_bitg():
    """ITfInputProcessorProfileMgr::ActivateProfile (vtable 3) — shu jarayon uchun bitg."""
    ole32.CoInitializeEx(None, 2)
    clsid_profiles = GUID.parse("33C53A50-F456-4884-B049-85FD643ECFED")
    iid_mgr = GUID.parse("71c6e74c-0f28-11d8-a82a-00065b84435c")
    pm = ctypes.c_void_p()
    hr = ole32.CoCreateInstance(ctypes.byref(clsid_profiles), None, 1, ctypes.byref(iid_mgr), ctypes.byref(pm))
    if hr:
        return hr & 0xFFFFFFFF
    vtbl = ctypes.cast(ctypes.cast(pm, ctypes.POINTER(ctypes.c_void_p))[0], ctypes.POINTER(ctypes.c_void_p))
    proto = ctypes.WINFUNCTYPE(ctypes.c_long, ctypes.c_void_p, wintypes.DWORD, wintypes.WORD,
                               ctypes.POINTER(GUID), ctypes.POINTER(GUID), ctypes.c_void_p, wintypes.DWORD)
    activate = proto(vtbl[3])
    clsid = GUID.parse("e37d4f70-f82a-435e-af8b-82e264391062")
    prof = GUID.parse("bed48fc8-8690-4712-bbe0-1fd3284ce7b7")
    return activate(pm, 1, 0x0443, ctypes.byref(clsid), ctypes.byref(prof), None, 0x10000000) & 0xFFFFFFFF


# --- US tartibi: belgi -> skan-kod (katta harf = Shift) ---
SCAN = {}
for i, c in enumerate("`1234567890-="):
    SCAN[c] = [0x29, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D][i]
for row, start in (("qwertyuiop[]", 0x10), ("asdfghjkl;'", 0x1E), ("zxcvbnm,./", 0x2C)):
    for i, c in enumerate(row):
        SCAN[c] = start + i
SCAN[" "] = 0x39
SHIFTED = {"{": "[", "}": "]", ":": ";", '"': "'"}
SHIFT, BACK = 0x2A, 0x0E


def tap(code):
    scan(code)
    scan(code, up=True)


def type_keys(keys):
    """Bir tikda hammasi navbatga tushadi — tez yozish kabi: keyingi harflar IME tahriridan oldin turadi."""
    for c in keys:
        up = c.isupper() or c in SHIFTED
        base = SHIFTED.get(c, c.lower())
        if up:
            scan(SHIFT)
        tap(SCAN[base])
        if up:
            scan(SHIFT, up=True)


# --- Sozlamalar: IME %APPDATA%\bitg ni sekundiga bir marta tekshiradi ---
cfg_dir = os.path.join(os.environ["APPDATA"], "bitg")
os.makedirs(cfg_dir, exist_ok=True)


def settings(**over):
    v = dict(enabled=1, auto_convert=1, keymap=0, layout="tilmoch", undo_backspace=1, undo_remember=0, fallback=1)
    v.update(over)
    text = "\n".join(f"{k}={x}" for k, x in v.items()) + "\ndisabled_apps=\n"
    with open(os.path.join(cfg_dir, "settings.ini"), "w", encoding="utf-8", newline="\n") as f:
        f.write(text)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="bitg-e2e-qt.txt")
    out = ap.parse_args().out

    ex = os.path.join(cfg_dir, "exceptions.txt")
    if os.path.exists(ex):
        os.remove(ex)
    settings()

    app = QApplication(sys.argv)
    box = QTextEdit()
    box.setWindowTitle("bitg e2e qt")
    box.setFont(QFont("Segoe UI", 14))
    box.resize(700, 180)
    log = [f"ui=qt python={sys.version.split()[0]} bits={ctypes.sizeof(ctypes.c_void_p) * 8}"]

    def focus():
        box.show()
        box.raise_()
        box.activateWindow()
        user32.SetForegroundWindow(int(box.winId()))
        box.setFocus()
        log.append(f"foreground={user32.GetForegroundWindow() == int(box.winId())}")

    def check(name, want):
        got = box.toPlainText()
        log.append(f"PASS {name}" if got == want else f"FAIL {name} got=[{got}] want=[{want}]")

    def clear():
        # Dastur tozalashini zaxira yo'l ko'rmaydi — shuning uchun oldingi qadam probel bilan tugaydi.
        box.clear()

    def w():
        pass

    # #41: parol maydoni (QLineEdit Password) — zaxira yo'l so'zni qayta yozmasligi kerak
    pw = QLineEdit()
    pw.setWindowTitle("bitg e2e qt password")
    pw.setFont(QFont("Segoe UI", 14))
    pw.resize(500, 60)

    def focus_pw():
        pw.show()
        pw.raise_()
        pw.activateWindow()
        user32.SetForegroundWindow(int(pw.winId()))
        pw.setFocus()
        log.append(f"foreground pw={user32.GetForegroundWindow() == int(pw.winId())}")

    def check_pw(name, want):
        got = pw.text()
        log.append(f"PASS {name}" if got == want else f"FAIL {name} got=[{got}] want=[{want}]")

    def echo(mode):
        return lambda: pw.setEchoMode(mode)

    fresh = QLineEdit()
    fresh.setWindowTitle("bitg e2e qt line")
    fresh.setFont(QFont("Segoe UI", 14))
    fresh.resize(500, 60)

    def focus_fresh():
        fresh.show()
        fresh.raise_()
        fresh.activateWindow()
        user32.SetForegroundWindow(int(fresh.winId()))
        fresh.setFocus()
        log.append(f"foreground line={user32.GetForegroundWindow() == int(fresh.winId())}")

    acts = {}

    def act(tag):
        def run():
            hr = activate_bitg()
            acts[tag] = hr
            log.append(f"ActivateProfile {tag} hr={hr:#x}")
        return run

    # tsf.log: shu jarayonning (pid) "activate"/"deactivate" qatorlari belgidan keyin
    tsf_log = os.path.join(os.environ.get("LOCALAPPDATA", ""), "bitg", "tsf.log")
    marks = {}

    def mark(tag):
        def run():
            marks[tag] = os.path.getsize(tsf_log) if os.path.exists(tsf_log) else 0
        return run

    def tsf_events(tag):
        if not os.path.exists(tsf_log):
            return []
        start = marks.get(tag, 0)
        if os.path.getsize(tsf_log) < start:
            start = 0  # log 512 KB dan oshib qayta boshlangan
        with open(tsf_log, "rb") as f:
            f.seek(start)
            lines = f.read().decode("utf-8", "replace").splitlines()
        me = f" pid={os.getpid()} "
        return [ln.rsplit(" ", 1)[-1] for ln in lines
                if me in ln and (ln.endswith(" activate") or ln.endswith(" deactivate"))]

    def require_active(tag, name):
        """'untouched' tekshiruvi bitg shu oynada faol bo'lgandagina ma'noli: ActivateProfile S_OK va
        belgidan keyingi oxirgi hodisa deactivate emas (CI VM fokusda profilni qaytarib yuborardi)."""
        def run():
            hr, ev = acts.get(tag), tsf_events(tag)
            ok = hr == 0 and (not ev or ev[-1] == "activate")
            log.append(f"PASS {name}" if ok else f"FAIL {name} hr={hr} tsf_events={ev}")
        return run

    def check_fresh(name, want):
        got = fresh.text()
        log.append(f"PASS {name}" if got == want else f"FAIL {name} got=[{got}] want=[{want}]")

    steps = [
        focus,
        lambda: log.append(f"ActivateProfile hr={activate_bitg():#x}"),
        focus, w, w,
        # 1) Birinchi so'z (fokusdan keyin hech narsa bosilmagan)
        lambda: type_keys("shahar "), w, w, lambda: check("first word converted", "şahar "),
        # 2) Darhol Backspace: asl so'z qaytadi (o'z Backspace'larimizni foydalanuvchiniki deb bilmaydi)
        lambda: tap(BACK), w, w, lambda: check("undo: backspace restores", "shahar"),
        lambda: type_keys(" "), w, lambda: check("undo: word kept on next space", "shahar "),
        # 3) Apostrofli va inglizcha so'z
        clear, lambda: type_keys("o'zbek "), w, w, lambda: check("apostrophe: o'zbek", "özbek "),
        clear, lambda: type_keys("check "), w, w, lambda: check("english word kept", "check "),
        # 4) Tez yozish: keyingi harflar tahririmizdan oldin navbatda turadi
        clear, lambda: type_keys("shoshma ok "), w, w, w, lambda: check("typed ahead: shoshma ok", "şoşma ok "),
        clear, lambda: type_keys("shoshma nima gap qo'l shapaloq "), w, w, w, w,
        lambda: check("typed ahead: sentence", "şoşma nima gap qöl şapaloq "),
        # 5) So'z ichida Backspace bilan tuzatish
        clear, lambda: type_keys("choz"), lambda: tap(BACK), lambda: type_keys("y "), w, w,
        lambda: check("backspace inside word", "çoy "),
        # 6) O'girilmagan so'zdan keyin Backspace odatdagidek
        clear, lambda: type_keys("nima "), w, lambda: tap(BACK), w, lambda: check("plain backspace", "nima"),
        # 7) Tilmoch tartibi ([ ; ' = ö ş ç). Sozlama >1 s da qayta o'qiladi; "a " uni shu tugmada oladi.
        lambda: type_keys(" "), lambda: settings(keymap=1, layout="tilmoch", auto_convert=0), w, w, w, w,
        lambda: type_keys("a "), w,
        clear, lambda: type_keys("[;' "), w, w, lambda: check("keymap: tilmoch letters", "öşç "),
        lambda: settings(keymap=1, layout="tilmoch", auto_convert=1), w, w, w, w,
        lambda: type_keys("a "), w,
        # Tartib harfi so'z kuzatuviga kiradi (Qt belgini bermaydi — navbat bo'yicha): sh keyin o'giriladi
        clear, lambda: type_keys("q[shimcha "), w, w, w, lambda: check("keymap: mapped letter tracked", "qöşimça "),
        lambda: settings(), w, w, w, w, lambda: type_keys("a "), w,
        # CI VM'da har yuqori oyna fokusida Windows profilni qaytaradi (tsf.log: "deactivate"), shuning uchun
        # har fokusdan keyin bitg qayta tanlanadi; "untouched" tekshiruvi oldidan va keyin shu oynada nazorat
        # so'zi o'giriladi — aks holda u bitg ishlamay turib ham o'tib ketadi.
        # 8) Qt maydon turini TSF'ga bermaydi (tsf.log: "vt=0 no scopes"): fokus o'zgarmasdan parolga aylangan
        #    maydonni zaxira yo'l sezmaydi. Ma'lum cheklov (#41) — natija faqat yoziladi, sinov yiqilmaydi.
        echo(QLineEdit.EchoMode.Normal), focus_pw, act("pw"), focus_pw, w, w,
        lambda: type_keys("choy "), w, w, w, lambda: check_pw("line edit converts (bitg active)", "çoy "),
        echo(QLineEdit.EchoMode.Password), w,
        lambda: type_keys("shoshma "), w, w, w,
        lambda: log.append(f"INFO field turned password without refocus (known limit, #41): got=[{pw.text()}]"),
        # 9) #41: fokus olayotgan parol maydoni — so'zlar qayta yozilmaydi
        mark("password"),
        lambda: fresh.setEchoMode(QLineEdit.EchoMode.Password), focus_fresh, act("password"), focus_fresh, w, w,
        lambda: type_keys("choy shoshma "), w, w, w,
        require_active("password", "password field: bitg active in this window"),
        lambda: check_fresh("password field untouched", "choy shoshma "),
        # Nazorat: o'sha oyna oddiy rejimda qayta fokus olsa bitg ishlaydi
        lambda: fresh.setEchoMode(QLineEdit.EchoMode.Normal), focus, w, focus_fresh, act("line"), focus_fresh, w, w,
        lambda: type_keys("choy "), w, w, w,
        lambda: check_fresh("same field back to normal converts (bitg active)", "choy shoshma çoy "),
    ]

    state = {"k": 0}

    def tick():
        k = state["k"]
        if k < len(steps):
            try:
                steps[k]()
            except Exception as e:  # noqa: BLE001
                log.append(f"ERROR step {k}: {e}")
            state["k"] = k + 1
        else:
            timer.stop()
            with open(out, "w", encoding="utf-8") as f:
                f.write("\n".join(log) + "\n")
            app.quit()

    timer = QTimer()
    timer.setInterval(350)
    timer.timeout.connect(tick)
    box.show()
    timer.start()
    app.exec()

    with open(out, encoding="utf-8") as f:
        text = f.read()
    sys.stdout.buffer.write(text.encode("utf-8"))
    sys.exit(1 if any(line.startswith(("FAIL", "ERROR")) for line in text.splitlines()) else 0)


if __name__ == "__main__":
    main()
