#!/usr/bin/python3
# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0
"""bitg StatusNotifier indicator for Ubuntu/GNOME (Ayatana AppIndicator)."""
import fcntl
import os
from pathlib import Path
import sys
import tempfile
import webbrowser

import gi
gi.require_version("Gtk", "3.0")
gi.require_version("AyatanaAppIndicator3", "0.1")
from gi.repository import Gtk, GLib, AyatanaAppIndicator3 as Indicator

KEYS = ("enabled", "auto_convert", "cyrillic")
LABELS = {"enabled": "Yoqilgan", "auto_convert": "Avtomatik ögiriş",
          "cyrillic": "Kirillçani ögiriş"}
CONFIG = Path(os.environ.get("XDG_CONFIG_HOME") or Path.home() / ".config") / "bitg"
SETTINGS = CONFIG / "settings.ini"
EXCEPTIONS = CONFIG / "exceptions.txt"
# crates/bitg-ime/src/config.rs `flag()` bilan bir xil
TRUE = ("1", "true", "yes", "on", "ha")
FALSE = ("0", "false", "no", "off", "yoq", "yo'q", "yo‘q")
BOM = "﻿"


def flag(value):
    value = value.strip().lower()
    if value in TRUE:
        return True
    if value in FALSE:
        return False
    return None


def read_text(path):
    try:
        return path.read_text(encoding="utf-8").removeprefix(BOM)
    except FileNotFoundError:
        return ""


def read_settings():
    values = {key: True for key in KEYS}
    for line in read_text(SETTINGS).splitlines():
        line = line.strip()
        if "=" not in line or line.startswith(("#", ";", "[")):
            continue
        key, val = line.split("=", 1)
        key = key.strip().lower()
        if key in values and flag(val) is not None:
            values[key] = flag(val)
    return values


def write_atomic(path, text):
    CONFIG.mkdir(mode=0o700, parents=True, exist_ok=True)
    fd, name = tempfile.mkstemp(prefix=f".{path.stem}-", dir=CONFIG)
    try:
        os.fchmod(fd, 0o600)
        with os.fdopen(fd, "w", encoding="utf-8") as output:
            output.write(text)
            output.flush()
            os.fsync(output.fileno())
        os.replace(name, path)
    finally:
        if os.path.exists(name):
            os.unlink(name)


def save_setting(key, enabled):
    lines = read_text(SETTINGS).splitlines(keepends=True)
    new_line = f"{key}={'1' if enabled else '0'}\n"
    found = False
    for i, line in enumerate(lines):
        if "=" in line and line.split("=", 1)[0].strip().lower() == key:
            lines[i] = "" if found else new_line
            found = True
    if not found:
        if lines and not lines[-1].endswith("\n"):
            lines[-1] += "\n"
        lines.append(new_line)
    write_atomic(SETTINGS, "".join(lines))


def error(text):
    dialog = Gtk.MessageDialog(message_type=Gtk.MessageType.ERROR,
                               buttons=Gtk.ButtonsType.CLOSE, text=text)
    dialog.run()
    dialog.destroy()


class App:
    def __init__(self):
        self.indicator = Indicator.Indicator.new(
            "bitg", "bitg", Indicator.IndicatorCategory.APPLICATION_STATUS)
        self.indicator.set_status(Indicator.IndicatorStatus.ACTIVE)
        self.menu = Gtk.Menu()
        self.items = {}
        for key in KEYS:
            item = Gtk.CheckMenuItem(label=LABELS[key])
            item.connect("toggled", self.toggle, key)
            self.menu.append(item)
            self.items[key] = item
        self.menu.append(Gtk.SeparatorMenuItem())
        settings = Gtk.MenuItem(label="Istisnolar va lugʼat…")
        settings.connect("activate", self.show_settings)
        self.menu.append(settings)
        report = Gtk.MenuItem(label="Xato haqida xabar beriş")
        report.connect("activate", lambda *_: webbrowser.open(
            "https://github.com/dtwinuz/bitg/issues/new"))
        self.menu.append(report)
        quit_item = Gtk.MenuItem(label="Panel belgisini yopiş")
        quit_item.connect("activate", lambda *_: Gtk.main_quit())
        self.menu.append(quit_item)
        self.menu.show_all()
        self.indicator.set_menu(self.menu)
        self.dialog = None
        self.sync()
        GLib.timeout_add_seconds(2, self.sync)

    def sync(self):
        values = read_settings()
        for key, item in self.items.items():
            item.handler_block_by_func(self.toggle)
            item.set_active(values[key])
            item.handler_unblock_by_func(self.toggle)
        self.indicator.set_icon_full(
            "bitg" if values["enabled"] else "bitg-disabled",
            "bitg yoqilgan" if values["enabled"] else "bitg öçirilgan")
        return True

    def toggle(self, item, key):
        try:
            save_setting(key, item.get_active())
        except OSError as exc:
            error(f"Sozlamani saqlab bölmadi: {exc}")
        self.sync()

    def show_settings(self, *_):
        if self.dialog is not None:
            self.dialog.present()
            return
        dialog = Gtk.Dialog(title="bitg: istisnolar va lugʼat")
        dialog.set_default_size(420, 420)
        dialog.add_button("Bekor qiliş", Gtk.ResponseType.CANCEL)
        dialog.add_button("Saqlaş", Gtk.ResponseType.OK)
        box = dialog.get_content_area()
        box.set_spacing(8)
        box.set_border_width(12)
        hint = Gtk.Label(xalign=0, wrap=True, label=(
            "Har qatorda bitta söz: u ögirilmaydi. «söz=yozuv» — söz aynan şunday yoziladi "
            "(masalan, şamollaş). # bilan boşlangan qator — izoh. Saqlangaç dasturlarni "
            "qayta oçmasdan qöllanadi."))
        box.pack_start(hint, False, False, 0)
        view = Gtk.TextView(monospace=True)
        view.get_buffer().set_text(read_text(EXCEPTIONS))
        scroll = Gtk.ScrolledWindow(hexpand=True, vexpand=True)
        scroll.add(view)
        box.pack_start(scroll, True, True, 0)
        dialog.connect("response", self.on_settings_response, view)
        self.dialog = dialog
        dialog.show_all()

    def on_settings_response(self, dialog, response, view):
        if response == Gtk.ResponseType.OK:
            buf = view.get_buffer()
            text = buf.get_text(buf.get_start_iter(), buf.get_end_iter(), False)
            if text and not text.endswith("\n"):
                text += "\n"
            try:
                write_atomic(EXCEPTIONS, text)
            except OSError as exc:
                error(f"Istisnolarni saqlab bölmadi: {exc}")
                return
        dialog.destroy()
        self.dialog = None


def main():
    runtime = Path(os.environ.get("XDG_RUNTIME_DIR") or f"/tmp/bitg-{os.getuid()}")
    runtime.mkdir(mode=0o700, parents=True, exist_ok=True)
    lock = open(runtime / "bitg-indicator.lock", "w")
    try:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        return 0
    App()
    Gtk.main()
    return 0


if __name__ == "__main__":
    sys.exit(main())
