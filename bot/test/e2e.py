#!/usr/bin/env python3
"""@bitguzbot uchidan-uchiga sinovi: soxta Telegram API (API_URL) bilan haqiqiy bot binari.

  python3 bot/test/e2e.py target/release/bitg-bot

Ssenariy: /start fikr -> rasmli fikr adminga nusxa bo'lib boradi -> admin reply qiladi -> javob foydalanuvchiga;
/fikr matn bilan; /bekor; guruhda /fikr; /yuklash; oddiy o'girish; bitta admin botni bloklagan bo'lsa ham yetkaziladi.
"""
import json
import os
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ADMIN, BLOCKED, USER, USER2, GROUP = 9001, 9002, 501, 502, -100777
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
        body = json.loads(self.rfile.read(int(self.headers.get("Content-Length", 0))) or b"{}")
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
        self.send_response(200 if ok else 403)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        try:
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
    env = {**os.environ, "BOT_TOKEN": "1:TEST", "API_URL": f"http://127.0.0.1:{srv.server_port}",
           "ADMIN_IDS": f"{BLOCKED},{ADMIN}", "NEWS_CHANNEL": "@bitguz", "SITE_URL": "https://bitg.uz"}
    bot = subprocess.Popen([exe], env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
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

        print(f"bot e2e: OK ({len(calls)} chaqiruv)")
    finally:
        bot.terminate()
        out = bot.communicate(timeout=5)[0]
        srv.shutdown()
        if "panic" in out:
            print(out)
            sys.exit(1)


if __name__ == "__main__":
    main()
