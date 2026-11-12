#!/usr/bin/env python3
"""ibus-daemon + o'rnatilgan bitg dvigateli bilan uchidan-uchiga sinov.

Dastur o'rnida IBus kiritish konteksti: tugmalar daemon orqali dvigatelga boradi, commit/delete/preedit
signallari matn modeliga qo'llanadi. Ishga tushirish (deb o'rnatilgandan keyin):

    dbus-run-session -- python3 linux/test/ibus_e2e.py
"""
import os
import subprocess
import sys
import time

import gi

gi.require_version("IBus", "1.0")
from gi.repository import GLib, IBus  # noqa: E402

KEYS = {" ": IBus.KEY_space, "\n": IBus.KEY_Return, "\t": IBus.KEY_Tab, "\b": IBus.KEY_BackSpace,
        "\x1b": IBus.KEY_Left}


def pump(sec=0.05):
    end = time.monotonic() + sec
    ctx = GLib.MainContext.default()
    while time.monotonic() < end:
        while ctx.iteration(False):
            pass
        time.sleep(0.005)


class App:
    """Matn maydoni: kursor doim oxirda."""

    def __init__(self, bus, surrounding=True, purpose=None):
        self.text = ""
        self.preedit = ""
        self.surrounding = surrounding
        self.ctx = bus.create_input_context("bitg-e2e")
        caps = IBus.Capabilite.FOCUS | IBus.Capabilite.PREEDIT_TEXT
        if surrounding:
            caps |= IBus.Capabilite.SURROUNDING_TEXT
        self.ctx.set_capabilities(caps)
        self.ctx.connect("commit-text", lambda _c, t: self._commit(t.get_text()))
        self.ctx.connect("delete-surrounding-text", lambda _c, off, n: self._delete(off, n))
        self.ctx.connect("update-preedit-text", lambda _c, t, _cur, vis: self._preedit(t.get_text() if vis else ""))
        self.ctx.connect("hide-preedit-text", lambda _c: self._preedit(""))
        self.ctx.focus_in()
        self.ctx.set_engine("bitg")
        if purpose is not None:
            self.ctx.set_content_type(purpose, 0)
        pump(0.3)

    def _commit(self, s):
        self.text += s

    def _delete(self, off, n):
        assert off == -n, (off, n)
        self.text = self.text[: len(self.text) - n]

    def _preedit(self, s):
        self.preedit = s

    def send_surrounding(self):
        if self.surrounding:
            n = len(self.text)
            self.ctx.set_surrounding_text(IBus.Text.new_from_string(self.text), n, n)

    def typ(self, s):
        for ch in s:
            keyval = KEYS.get(ch) or IBus.unicode_to_keyval(ch)  # GI: gunichar = str
            self.send_surrounding()
            pump(0.01)
            consumed = self.ctx.process_key_event(keyval, 0, 0)
            pump(0.03)
            self.ctx.process_key_event(keyval, 0, IBus.ModifierType.RELEASE_MASK)
            if not consumed:  # dastur tugmani o'zi qayta ishlaydi
                if ch == "\b":
                    self.text = self.text[:-1]
                elif ch != "\x1b":
                    self.text += ch
        pump(0.05)
        return self

    def close(self):
        self.ctx.focus_out()
        pump(0.1)
        self.ctx.destroy()


CASES = []


def case(fn):
    CASES.append(fn)
    return fn


def check(name, got, want):
    if got != want:
        raise AssertionError(f"{name}: {got!r} != {want!r}")


@case
def direct_converts(bus):
    a = App(bus).typ("keyin choy ichamiz ")
    check("text", a.text, "keyin çoy içamiz ")
    check("preedit", a.preedit, "")
    a.close()


@case
def english_untouched(bus):
    a = App(bus).typ("Please check the schedule, keyin choy ichamiz ")
    check("text", a.text, "Please check the schedule, keyin çoy içamiz ")
    a.close()


@case
def cyrillic(bus):
    a = App(bus).typ("чой ичамиз ")
    check("text", a.text, "çoy içamiz ")
    a.close()


@case
def backspace_undo(bus):
    a = App(bus).typ("choy ")
    check("converted", a.text, "çoy ")
    a.typ("\b")
    check("restored", a.text, "choy")
    a.typ(" choy ")
    check("learned", a.text, "choy choy ")
    a.close()


@case
def enter_keeps_newline(bus):
    a = App(bus).typ("salom shahar\n")
    check("text", a.text, "salom şahar\n")
    a.close()


@case
def preedit_mode(bus):
    a = App(bus, surrounding=False).typ("shahar")
    check("held", (a.text, a.preedit), ("", "shahar"))
    a.typ(" choy\n")
    check("text", a.text, "şahar çoy\n")
    check("preedit", a.preedit, "")
    a.close()


@case
def preedit_kept_on_focus_out(bus):
    a = App(bus, surrounding=False).typ("choy")
    a.ctx.focus_out()
    pump(0.2)
    check("committed raw", a.text, "choy")
    a.ctx.destroy()


@case
def password_field_untouched(bus):
    a = App(bus, purpose=IBus.InputPurpose.PASSWORD).typ("choy shahar ")
    check("text", a.text, "choy shahar ")
    a.close()


def main():
    log_path = os.environ.get("BITG_IBUS_LOG", "/tmp/bitg-ibus-daemon.log")
    log = open(log_path, "wb")
    daemon = None
    if "--no-daemon" not in sys.argv:
        # X display yo'q: manzil fayldan emas, aniq beriladi (daemon dvigatelga ham shuni uzatadi)
        address = f"unix:path=/tmp/bitg-ibus-{os.getpid()}"
        os.environ["IBUS_ADDRESS"] = address
        daemon = subprocess.Popen(["ibus-daemon", "--panel=disable", "--emoji-extension=disable", "--replace",
                                   "--verbose", f"--address={address}"], stdout=log, stderr=subprocess.STDOUT)
    IBus.init()
    # IBus.Bus.new() singleton: daemon tayyor bo'lmasdan yaratilsa, keyin ham ulanmagan obyekt qaytadi.
    # Shuning uchun avval manzil ulanishni qabul qilguncha kutamiz, keyin bus'ni bir marta yaratamiz.
    from gi.repository import Gio
    addr = os.environ.get("IBUS_ADDRESS") or IBus.get_address()
    for _ in range(100):
        try:
            c = Gio.DBusConnection.new_for_address_sync(
                addr, Gio.DBusConnectionFlags.AUTHENTICATION_CLIENT | Gio.DBusConnectionFlags.MESSAGE_BUS_CONNECTION,
                None, None)
            c.close_sync(None)
            break
        except Exception:  # daemon hali tinglamayapti
            time.sleep(0.1)
    bus = IBus.Bus.new()
    for _ in range(50):
        if bus.is_connected():
            break
        pump(0.1)
    if bus is None or not bus.is_connected():
        log.close()
        with open(log_path, "rb") as f:
            print(f.read().decode(errors="replace")[-3000:])
        print("address:", addr, "| daemon alive:", daemon is not None and daemon.poll() is None)
        sys.exit("ibus-daemon'ga ulanib bo'lmadi")
    for _ in range(50):  # komponentlar ro'yxati yuklanguncha
        if any(e.get_name() == "bitg" for e in bus.list_engines()):
            break
        time.sleep(0.1)
    names = [e.get_name() for e in bus.list_engines()]
    if "bitg" not in names:
        sys.exit(f"bitg dvigateli ro'yxatda yo'q (component o'rnatilmaganmi?): {len(names)} ta dvigatel")
    failed = 0
    for fn in CASES:
        try:
            fn(bus)
            print(f"ok   {fn.__name__}")
        except Exception as e:  # sinov hisobotida hammasi ko'rinsin
            failed += 1
            print(f"FAIL {fn.__name__}: {e}")
    if daemon:
        daemon.terminate()
        try:
            daemon.wait(timeout=5)
        except subprocess.TimeoutExpired:
            daemon.kill()
    log.close()
    if failed:
        with open(log_path, "rb") as f:
            print("--- ibus-daemon log (oxiri) ---")
            print(f.read().decode(errors="replace")[-3000:])
    print(f"{len(CASES) - failed}/{len(CASES)} o'tdi")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    os.environ.setdefault("LANG", "C.UTF-8")
    main()
