#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

"""@bitguzbot uchidan-uchiga sinovi: soxta Telegram API (API_URL) bilan haqiqiy bot binari.

  python3 bot/test/e2e.py target/release/bitg-bot

Ssenariy: /start fikr -> rasmli fikr adminga nusxa bo'lib boradi -> admin reply qiladi -> javob foydalanuvchiga;
/fikr matn bilan; /bekor; guruhda /fikr; /yuklash; oddiy o'girish; bitta admin botni bloklagan bo'lsa ham yetkaziladi;
/fikr dan keyin bot qayta ishga tushsa ham fikr yetkaziladi (STATE_DIR), tiriklik belgisi yoziladi.
"""
import json
import os
import socket
import urllib.error
import urllib.request
import shutil
import subprocess
import tempfile
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ADMIN, BLOCKED, USER, USER2, USER3, GROUP = 9001, 9002, 501, 502, 503, -100777
calls = []          # (method, body, result)
lock = threading.Lock()
mid = [1000]
queue = []          # navbatdagi update'lar


def new_id():
    with lock:
        mid[0] += 1
        return mid[0]


def msg(chat, text=None, chat_type="private", **kw):
    m = {"message_id": new_id(), "chat": {"id": chat, "type": chat_type},
         "from": {"id": abs(chat), "first_name": "Ali", "username": f"u{abs(chat)}"}, "date": 0}
    if text is not None:
        m["text"] = text
    m.update(kw)
    return m


class H(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def do_POST(self):
        method = self.path.rsplit("/", 1)[-1]
        raw = self.rfile.read(int(self.headers.get("Content-Length", 0)))
        if self.headers.get("Content-Type", "").startswith("multipart/form-data"):
            # sendPhoto: chat_id va fayl borligini tekshirish kifoya
            chat = raw.split(b'name="chat_id"\r\n\r\n', 1)[1].split(b"\r\n", 1)[0]
            body = {"chat_id": int(chat), "multipart": True, "photo": b'name="photo"' in raw}
        else:
            body = json.loads(raw or b"{}")
        ok, result = True, True
        if method == "getMe":
            result = {"id": 1, "is_bot": True, "username": "bitguzbot"}
        elif method == "getUpdates":
            with lock:
                ups = list(queue)
                queue.clear()
            if not ups:
                time.sleep(0.2)
            result = [{"update_id": new_id(), "message": u} for u in ups]
        elif method in ("sendMessage", "copyMessage"):
            if body.get("chat_id") == BLOCKED:
                ok = False
            else:
                result = {"message_id": new_id()}
        resp = {"ok": True, "result": result} if ok else {"ok": False, "description": "Forbidden: bot was blocked by the user"}
        if method != "getUpdates":
            with lock:
                calls.append((method, body, resp.get("result")))
        data = json.dumps(resp).encode()
        try:
            self.send_response(200 if ok else 403)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)
        except OSError:
            pass  # sinov oxirida bot long-poll paytida to'xtatiladi


def push(m):
    with lock:
        queue.append(m)


def wait_for(pred, what, timeout=10):
    end = time.time() + timeout
    while time.time() < end:
        with lock:
            hit = [c for c in calls if pred(c)]
        if hit:
            return hit
        time.sleep(0.05)
    with lock:
        tail = "\n".join(f"  {c[0]} {c[1]}" for c in calls[-12:])
    raise AssertionError(f"kutilgan chaqiruv yo'q: {what}\n{tail}")


def sent(chat, contains=None, method="sendMessage"):
    return lambda c: c[0] == method and c[1].get("chat_id") == chat and (
        contains is None or contains in c[1].get("text", ""))


def count_to(chat):
    with lock:
        return len([c for c in calls if c[1].get("chat_id") == chat])


def main():
    exe = sys.argv[1]
    srv = ThreadingHTTPServer(("127.0.0.1", 0), H)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    state = tempfile.mkdtemp(prefix="bitg-bot-state-")
    probe = socket.socket()
    probe.bind(("127.0.0.1", 0))
    api_port = probe.getsockname()[1]
    probe.close()
    env = {**os.environ, "BOT_TOKEN": "1:TEST", "API_URL": f"http://127.0.0.1:{srv.server_port}",
           "ADMIN_IDS": f"{BLOCKED},{ADMIN}", "NEWS_CHANNEL": "@bitguz", "SITE_URL": "https://bitg.uz",
           "STATE_DIR": state, "HTTP_PORT": str(api_port)}

    def api(method, path, body=None, ctype="application/json", xff="203.0.113.7"):
        data = json.dumps(body).encode() if isinstance(body, dict) else body
        req = urllib.request.Request(f"http://127.0.0.1:{api_port}{path}", data=data, method=method,
                                     headers={"Content-Type": ctype, "X-Forwarded-For": xff})
        try:
            with urllib.request.urlopen(req, timeout=10) as r:
                return r.status, json.loads(r.read())
        except urllib.error.HTTPError as e:
            return e.code, json.loads(e.read())

    def start():
        return subprocess.Popen([exe], env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)

    bot = start()
    logs = []
    try:
        wait_for(lambda c: c[0] == "setMyCommands", "setMyCommands")
        cmds = [c for c in calls if c[0] == "setMyCommands"][0][1]["commands"]
        assert {"fikr", "yuklash"} <= {c["command"] for c in cmds}, cmds

        # 1) /start fikr -> taklif; keyingi rasm fikr bo'ladi (bloklagan admin bo'lsa ham)
        push(msg(USER, "/start fikr"))
        wait_for(sent(USER, "bitta xabarda"), "fikr taklifi")
        photo = msg(USER, photo=[{"file_id": "F", "width": 1, "height": 1}], caption="Word'da ikki marta yozildi")
        push(photo)
        head = wait_for(sent(ADMIN, "💬 Fikr · Ali (@u501) · id 501"), "admin sarlavha")[0]
        copy = wait_for(lambda c: c[0] == "copyMessage" and c[1]["chat_id"] == ADMIN
                        and c[1]["from_chat_id"] == USER and c[1]["message_id"] == photo["message_id"], "admin nusxa")[0]
        assert copy[1]["reply_parameters"]["message_id"] == head[2]["message_id"]
        wait_for(sent(USER, "Rahmat!"), "foydalanuvchiga tasdiq")

        # 2) admin sarlavhaga reply qiladi -> foydalanuvchiga javob
        rep = msg(ADMIN, "Rahmat, tuzatamiz",
                  reply_to_message={"message_id": head[2]["message_id"], "text": head[1]["text"]})
        push(rep)
        wait_for(sent(USER, "bitg jamoasidan javob"), "javob sarlavhasi")
        wait_for(lambda c: c[0] == "copyMessage" and c[1]["chat_id"] == USER and c[1]["from_chat_id"] == ADMIN
                 and c[1]["message_id"] == rep["message_id"], "javob nusxasi")
        wait_for(sent(ADMIN, "✅ Yuborildi"), "adminga tasdiq")

        # 2b) bot qayta ishga tushgandan keyin ham: eslab qolinmagan xabar, lekin sarlavhada "id 501"
        rep2 = msg(ADMIN, "Yana bir savol", reply_to_message={"message_id": 1, "text": head[1]["text"]})
        push(rep2)
        wait_for(lambda c: c[0] == "copyMessage" and c[1]["chat_id"] == USER
                 and c[1]["message_id"] == rep2["message_id"], "sarlavhadan id")

        # 3) /fikr matn bilan (boshqa foydalanuvchi) -> darhol yetkaziladi; admin nusxaga reply qiladi
        push(msg(USER2, "/fikr Telegram'da ishlamayapti"))
        wait_for(sent(ADMIN, "id 502"), "2-fikr sarlavha")
        c2 = wait_for(lambda c: c[0] == "copyMessage" and c[1]["chat_id"] == ADMIN
                      and c[1]["from_chat_id"] == USER2, "2-fikr nusxa")[0]
        push(msg(ADMIN, "Versiyangiz qaysi?", reply_to_message={"message_id": c2[2]["message_id"]}))
        wait_for(lambda c: c[0] == "copyMessage" and c[1]["chat_id"] == USER2
                 and c[1]["from_chat_id"] == ADMIN, "nusxaga javob")

        # 3b) shu foydalanuvchi 20 soniya ichida yana yozsa -> kutish
        push(msg(USER2, "/fikr yana"))
        wait_for(sent(USER2, "Biroz kuting"), "spam himoyasi")

        # 4) /fikr, keyin /bekor -> keyingi matn oddiy o'girish
        push(msg(USER, "/fikr"))
        wait_for(lambda c: len([x for x in calls if x[0] == "sendMessage" and x[1].get("chat_id") == USER
                                and "bitta xabarda" in x[1].get("text", "")]) >= 2, "2-taklif")
        push(msg(USER, "/bekor"))
        wait_for(sent(USER, "Bekor qilindi"), "bekor")
        push(msg(USER, "O'zbekiston shaharlari"))
        wait_for(sent(USER, "Özbekiston şaharlari"), "o'girish")

        # 5) guruhda /fikr -> shaxsiy chatga yo'naltiradi; /yuklash tugmalar bilan
        push(msg(GROUP, "/fikr@bitguzbot", chat_type="supergroup"))
        wait_for(sent(GROUP, "t.me/bitguzbot?start=fikr"), "guruhda fikr")
        push(msg(USER, "/yuklash"))
        dl = wait_for(sent(USER, "brew install --cask dtwinuz/tap/bitg"), "yuklash")[0]
        assert len(dl[1]["reply_markup"]["inline_keyboard"][0]) == 3

        # 6) /start yordam: tugmalarda yangiliklar va fikr
        push(msg(USER, "/start"))
        h = wait_for(sent(USER, "/fikr"), "yordam")[-1]
        urls = [b.get("url", "") for row in h[1]["reply_markup"]["inline_keyboard"] for b in row]
        assert "https://t.me/bitguz" in urls and "https://t.me/bitguzbot?start=fikr" in urls, urls

        # 7) admin oddiy (reply bo'lmagan) xabar yozsa -> o'girish, foydalanuvchiga hech narsa ketmaydi
        n_user = count_to(USER)
        push(msg(ADMIN, "choy"))
        wait_for(sent(ADMIN, "çoy"), "admin o'girish")
        time.sleep(0.3)
        assert count_to(USER) == n_user

        # 7b) ilovadan fikr (HTTP_PORT): matn + rasm adminlarga; bloklagan admin bo'lsa ham yetadi
        assert api("GET", "/api/health") == (200, {"ok": True})
        fb = {"v": 1, "kind": "conversion", "typed": "shahar", "got": "shahar", "expected": "şahar",
              "image": "data:image/png;base64,iVBORw0KGgo=", "app": "windows", "version": "0.4.0",
              "id": "e2e-install-1", "diag": {"os": "10.0.26100"}}
        code, r = api("POST", "/api/feedback", fb)
        assert code == 200 and len(r["ticket"]) == 6, (code, r)
        wait_for(sent(ADMIN, f"Noto'g'ri o'girish · #{r['ticket']}"), "ilova fikri matni")
        wait_for(lambda c: c[0] == "sendPhoto" and c[1]["chat_id"] == ADMIN and c[1]["photo"], "ilova fikri rasmi")
        assert api("POST", "/api/feedback", {**fb, "kind": "x"}) == (400, {"ok": False, "error": "kind"})
        assert api("POST", "/api/feedback", b"{}", ctype="text/plain")[0] == 415
        assert api("GET", "/nope")[0] == 404
        for _ in range(4):
            assert api("POST", "/api/feedback", fb)[0] == 200
        code, r = api("POST", "/api/feedback", fb)
        assert code == 429 and r["error"] == "rate" and r["retry"] > 0, (code, r)
        assert api("POST", "/api/feedback", fb, xff="198.51.100.9")[0] == 200

        # 8) STATE_DIR: /fikr bosilgandan keyin bot qayta ishga tushsa ham keyingi xabar fikr bo'ladi;
        #    tiriklik belgisi (alive) yoziladi — bitg-stats shu fayl bilan kuzatadi
        push(msg(USER3, "/fikr"))
        wait_for(sent(USER3, "bitta xabarda"), "3-taklif")
        alive = os.path.join(state, "alive")
        assert int(open(alive, encoding="utf-8").read()) > 1_700_000_000
        assert "503" in open(os.path.join(state, "state.json"), encoding="utf-8").read()
        bot.terminate()
        logs.append(bot.communicate(timeout=5)[0])
        bot = start()
        wait_for(lambda c: c[0] == "setMyCommands" and len([x for x in calls if x[0] == "setMyCommands"]) >= 2,
                 "qayta ishga tushish")
        push(msg(USER3, "Restartdan keyin ham yetib bordi"))
        wait_for(sent(ADMIN, "id 503"), "tiklangan /fikr")
        # eslab qolingan nusxaga reply (sarlavhasiz) ham qayta ishga tushgandan keyin ishlaydi
        push(msg(ADMIN, "Ko'rdik", reply_to_message={"message_id": c2[2]["message_id"]}))
        wait_for(lambda c: c[0] == "copyMessage" and c[1]["chat_id"] == USER2 and c[1]["from_chat_id"] == ADMIN
                 and len([x for x in calls if x[0] == "copyMessage" and x[1]["chat_id"] == USER2]) >= 2,
                 "tiklangan suhbat")

        print(f"bot e2e: OK ({len(calls)} chaqiruv)")
    finally:
        bot.terminate()
        logs.append(bot.communicate(timeout=5)[0])
        srv.shutdown()
        shutil.rmtree(state, ignore_errors=True)
    out = "".join(logs)
    if "panic" in out or "holat tiklandi: 1 fikr kutilmoqda" not in out:
        print(out)
        sys.exit(1)


if __name__ == "__main__":
    main()
