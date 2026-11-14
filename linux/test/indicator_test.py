#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

"""O'rnatilgan yuqori panel belgisi (`/usr/bin/bitg-indicator`) sinovi (#75, #84).

Sozlama fayllari IME bilan bir xil o'qilishi/yozilishi va GUI smoke: belgi ishga tushadi, ikkinchi nusxa
ochilmaydi. X display kerak (Xvfb):

    xvfb-run -a dbus-run-session -- python3 linux/test/indicator_test.py
"""
import importlib.machinery
import importlib.util
import os
import stat
import subprocess
import sys
import tempfile
import time
import unittest

PATH = os.environ.get("BITG_INDICATOR", "/usr/bin/bitg-indicator")
os.environ["XDG_CONFIG_HOME"] = tempfile.mkdtemp(prefix="bitg-indicator-test-")
os.environ["XDG_RUNTIME_DIR"] = tempfile.mkdtemp(prefix="bitg-indicator-run-")
os.chmod(os.environ["XDG_RUNTIME_DIR"], 0o700)


def load():
    loader = importlib.machinery.SourceFileLoader("bitg_indicator", PATH)
    module = importlib.util.module_from_spec(importlib.util.spec_from_loader(loader.name, loader))
    loader.exec_module(module)
    return module


ind = load()


def write(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


class Settings(unittest.TestCase):
    def tearDown(self):
        for p in (ind.SETTINGS, ind.EXCEPTIONS):
            if p.exists():
                p.unlink()

    def test_defaults_when_missing(self):
        self.assertEqual(ind.read_settings(), {"enabled": True, "auto_convert": True, "cyrillic": True})

    def test_parses_like_ime(self):
        # crates/bitg-ime/src/config.rs `parses_windows_format` bilan bir xil kirish
        write(ind.SETTINGS, "﻿[bitg]\n# izoh\nEnabled = yo'q\nauto_convert=ha\ncyrillic=xato\nnoma'lum=1\n")
        self.assertEqual(ind.read_settings(), {"enabled": False, "auto_convert": True, "cyrillic": True})

    def test_save_keeps_other_lines_and_dedups(self):
        write(ind.SETTINGS, "[bitg]\ntutuq='\nenabled=1\nENABLED=1\nundo_remember=ha")
        ind.save_setting("enabled", False)
        text = ind.SETTINGS.read_text(encoding="utf-8")
        self.assertEqual(text, "[bitg]\ntutuq='\nenabled=0\nundo_remember=ha\n")
        self.assertFalse(ind.read_settings()["enabled"])

    def test_broken_utf8_is_not_overwritten(self):
        # buzilgan fayl: o'qishda standart qiymatlar, saqlash esa faylni ustidan yozmaydi
        ind.CONFIG.mkdir(parents=True, exist_ok=True)
        ind.SETTINGS.write_bytes(b"enabled=0\n\xff\xfe junk\n")
        self.assertIsNone(ind.read_text(ind.SETTINGS))
        self.assertEqual(ind.read_settings(), {"enabled": True, "auto_convert": True, "cyrillic": True})
        with self.assertRaises(OSError):
            ind.save_setting("enabled", True)
        self.assertEqual(ind.SETTINGS.read_bytes(), b"enabled=0\n\xff\xfe junk\n")

    def test_save_creates_private_files(self):
        ind.save_setting("cyrillic", False)
        self.assertEqual(stat.S_IMODE(ind.SETTINGS.stat().st_mode), 0o600)
        self.assertEqual(stat.S_IMODE(ind.CONFIG.stat().st_mode), 0o700)
        self.assertEqual(ind.SETTINGS.read_text(encoding="utf-8"), "cyrillic=0\n")
        leftovers = [p.name for p in ind.CONFIG.iterdir() if p.name.startswith(".")]
        self.assertEqual(leftovers, [], "vaqtinchalik fayl qolmasin")

    def test_exceptions_round_trip(self):
        ind.write_atomic(ind.EXCEPTIONS, "shahar\nshamollash=şamollaş\n")
        self.assertEqual(ind.read_text(ind.EXCEPTIONS), "shahar\nshamollash=şamollaş\n")
        write(ind.EXCEPTIONS, "﻿shahar\n")
        self.assertEqual(ind.read_text(ind.EXCEPTIONS), "shahar\n")


@unittest.skipUnless(os.environ.get("DISPLAY") or os.environ.get("WAYLAND_DISPLAY"), "X display yo'q")
class Smoke(unittest.TestCase):
    def test_runs_and_is_single_instance(self):
        first = subprocess.Popen([sys.executable, PATH], stderr=subprocess.PIPE)
        try:
            time.sleep(3)
            if first.poll() is not None:
                self.fail("belgi to'xtadi: " + first.stderr.read().decode(errors="replace")[-2000:])
            second = subprocess.run([sys.executable, PATH], timeout=10, capture_output=True)
            self.assertEqual(second.returncode, 0, second.stderr.decode(errors="replace"))
            self.assertIsNone(first.poll(), "ikkinchi nusxa birinchisini to'xtatmasin")
        finally:
            first.terminate()
            first.wait(timeout=10)


if __name__ == "__main__":
    unittest.main(verbosity=2)
