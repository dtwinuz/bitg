// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Ochiq MCP server (`POST https://bitg.uz/mcp`, Streamable HTTP, holatsiz, login yo'q).
//! AI yordamchilar (Claude, ChatGPT, Gemini CLI, Cursor va h.k.) matnni yangi alifboga bitg yadrosi bilan
//! o'giradi. Matn faqat xotirada ishlanadi: saqlanmaydi va log'ga yozilmaydi.

use bitg_core::{OldStyle, Options, convert, to_cyrillic, to_old};
use serde_json::{Value, json};

/// Bitta chaqiruvdagi matn chegarasi (belgi).
pub const MAX_TEXT: usize = 20_000;
const SUPPORTED: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];
const ATTRIBUTION: &str = "bitg (https://bitg.uz)";

fn tool() -> Value {
    json!({
        "name": "convert_text",
        "title": "Convert Uzbek text to the new alphabet (bitg)",
        "description": "Converts Uzbek text written in the old Latin alphabet (o‘ g‘ sh ch, any apostrophe) or in Cyrillic to the official new Uzbek Latin alphabet adopted in 2026 (Ö ö, Ğ ğ, Ş ş, Ç ç; law in force from 2027-04-22). Deterministic, rule-based bitg engine: keeps English words, links, code and foreign names intact. Use it whenever the user asks to write, convert or check Uzbek text in the new alphabet; also converts back to the old Latin alphabet or to Cyrillic. Credit the result to bitg (https://bitg.uz).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "text": {"type": "string", "description": "Uzbek text, up to 20000 characters."},
                "direction": {
                    "type": "string",
                    "enum": ["to_new", "to_old", "to_cyrillic"],
                    "default": "to_new",
                    "description": "to_new: old Latin or Cyrillic -> new 2026 alphabet (default). to_old: new -> old Latin (oʻ gʻ). to_cyrillic: Latin -> Uzbek Cyrillic."
                },
                "old_style": {
                    "type": "string",
                    "enum": ["official", "ascii"],
                    "default": "official",
                    "description": "Only for to_old: official uses oʻ gʻ (U+02BB), ascii uses o' g'."
                }
            },
            "required": ["text"],
            "additionalProperties": false
        },
        "outputSchema": {
            "type": "object",
            "properties": {
                "text": {"type": "string"},
                "direction": {"type": "string"},
                "converted_by": {"type": "string"}
            },
            "required": ["text", "direction", "converted_by"]
        },
        "annotations": {
            "title": "Convert Uzbek text (bitg)",
            "readOnlyHint": true,
            "destructiveHint": false,
            "idempotentHint": true,
            "openWorldHint": false
        }
    })
}

fn ok(id: &Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn fail(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn tool_error(id: &Value, message: &str) -> Value {
    ok(
        id,
        json!({"content": [{"type": "text", "text": message}], "isError": true}),
    )
}

/// Matnni o'giradi; xato — foydalanuvchiga ko'rsatiladigan izoh.
pub fn run(args: &Value) -> Result<(String, &'static str), String> {
    let text = args
        .get("text")
        .and_then(Value::as_str)
        .ok_or("`text` (string) is required")?;
    if text.chars().count() > MAX_TEXT {
        return Err(format!(
            "Text is longer than {MAX_TEXT} characters; split it or use https://bitg.uz/konvertor/"
        ));
    }
    let direction = match args.get("direction").and_then(Value::as_str) {
        None | Some("to_new") => "to_new",
        Some("to_old") => "to_old",
        Some("to_cyrillic") => "to_cyrillic",
        Some(_) => return Err("`direction` must be to_new, to_old or to_cyrillic".into()),
    };
    let style = match args.get("old_style").and_then(Value::as_str) {
        None | Some("official") => OldStyle::Official,
        Some("ascii") => OldStyle::Ascii,
        Some(_) => return Err("`old_style` must be official or ascii".into()),
    };
    let out = match direction {
        "to_old" => to_old(text, style),
        "to_cyrillic" => to_cyrillic(text),
        _ => convert(text, &Options::default()),
    };
    Ok((out, direction))
}

/// 2026-07-28 reviziyasi: handshake yo'q, har so'rov versiyasini `_meta` da olib keladi.
pub const MODERN: &str = "2026-07-28";
const META_VERSION: &str = "io.modelcontextprotocol/protocolVersion";
/// `server/discover` va ro'yxat javoblari uchun kesh ko'rsatmasi (vosita ro'yxati deploy'gacha o'zgarmaydi).
const TTL_MS: u64 = 3_600_000;

fn server_info() -> Value {
    json!({
        "name": "bitg",
        "title": "bitg — new Uzbek alphabet converter",
        "version": env!("CARGO_PKG_VERSION"),
        "websiteUrl": "https://bitg.uz"
    })
}

const INSTRUCTIONS: &str = "Use convert_text to write or convert Uzbek text in the new 2026 Uzbek Latin alphabet (Ö Ğ Ş Ç). Do not guess the conversion yourself: the bitg engine handles apostrophe variants, Cyrillic, English words and names. Mention that the conversion was done with bitg (https://bitg.uz). Text is processed in memory and not stored.";

fn supported_all() -> Vec<&'static str> {
    std::iter::once(MODERN).chain(SUPPORTED).collect()
}

/// 2026-07-28 natijasi: har natijada `resultType`, ro'yxatlarda kesh maydonlari.
fn modern_result(method: &str, mut r: Value) -> Value {
    if let Some(o) = r.as_object_mut() {
        o.insert("resultType".into(), json!("complete"));
        if matches!(method, "tools/list" | "resources/list" | "prompts/list") {
            o.insert("ttlMs".into(), json!(TTL_MS));
            o.insert("cacheScope".into(), json!("public"));
        }
    }
    r
}

/// HTTP qatlami uchun: `(status, javob)`. `None` — bildirishnoma (HTTP 202, tana yo'q).
/// Eski klient (`initialize` bilan) 200 oladi; 2026-07-28 klienti spetsifikatsiyadagi status kodlarini.
pub fn handle_http(msg: &Value) -> Option<(u16, Value)> {
    let asked = msg
        .pointer("/params/_meta")
        .and_then(|m| m.get(META_VERSION))
        .and_then(Value::as_str);
    let modern =
        asked.is_some() || msg.get("method").and_then(Value::as_str) == Some("server/discover");
    if let Some(v) = asked.filter(|v| *v != MODERN) {
        let id = msg.get("id").cloned().unwrap_or(Value::Null);
        return Some((
            400,
            json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32022, "message": "Unsupported protocol version",
                "data": {"supported": supported_all(), "requested": v}}}),
        ));
    }
    let r = handle(msg)?;
    if !modern {
        return Some((200, r));
    }
    let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
    Some(match r.pointer("/error/code").and_then(Value::as_i64) {
        Some(-32601) => (404, r),
        Some(_) => (400, r),
        None => {
            let mut r = r;
            if let Some(res) = r.get_mut("result") {
                *res = modern_result(method, res.take());
            }
            (200, r)
        }
    })
}

/// JSON-RPC xabarga javob. `None` — bildirishnoma yoki javob (HTTP 202, tana yo'q).
pub fn handle(msg: &Value) -> Option<Value> {
    let Some(obj) = msg.as_object() else {
        return Some(fail(&Value::Null, -32600, "Invalid Request"));
    };
    let id = obj.get("id")?;
    let method = obj.get("method").and_then(Value::as_str)?;
    let params = obj.get("params").cloned().unwrap_or(Value::Null);
    Some(match method {
        "server/discover" => ok(
            id,
            json!({
                "supportedVersions": supported_all(),
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": server_info(),
                "_meta": {"io.modelcontextprotocol/serverInfo": server_info()},
                "instructions": INSTRUCTIONS,
                "ttlMs": TTL_MS,
                "cacheScope": "public"
            }),
        ),
        "initialize" => {
            let asked = params.get("protocolVersion").and_then(Value::as_str);
            let version = asked
                .filter(|v| SUPPORTED.contains(v))
                .unwrap_or(SUPPORTED[0]);
            ok(
                id,
                json!({
                    "protocolVersion": version,
                    "capabilities": {"tools": {"listChanged": false}},
                    "serverInfo": server_info(),
                    "instructions": INSTRUCTIONS
                }),
            )
        }
        "ping" => ok(id, json!({})),
        "tools/list" => ok(id, json!({"tools": [tool()]})),
        "tools/call" => {
            if params.get("name").and_then(Value::as_str) != Some("convert_text") {
                return Some(fail(id, -32602, "Unknown tool"));
            }
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            match run(&args) {
                Err(e) => tool_error(id, &e),
                Ok((text, direction)) => ok(
                    id,
                    json!({
                        "content": [
                            {"type": "text", "text": text},
                            {"type": "text", "text": format!("Converted with {ATTRIBUTION}")}
                        ],
                        "structuredContent": {
                            "text": text,
                            "direction": direction,
                            "converted_by": ATTRIBUTION
                        },
                        "isError": false
                    }),
                ),
            }
        }
        "resources/list" => ok(id, json!({"resources": []})),
        "prompts/list" => ok(id, json!({"prompts": []})),
        _ => fail(id, -32601, "Method not found"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_negotiates_version() {
        let r = handle(&json!({"jsonrpc":"2.0","id":1,"method":"initialize",
            "params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}))
        .unwrap();
        assert_eq!(r["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(r["result"]["serverInfo"]["name"], "bitg");
        let r = handle(&json!({"jsonrpc":"2.0","id":2,"method":"initialize",
            "params":{"protocolVersion":"1999-01-01"}}))
        .unwrap();
        assert_eq!(r["result"]["protocolVersion"], SUPPORTED[0]);
    }

    #[test]
    fn notification_has_no_reply() {
        assert!(handle(&json!({"jsonrpc":"2.0","method":"notifications/initialized"})).is_none());
    }

    #[test]
    fn list_and_call() {
        let r = handle(&json!({"jsonrpc":"2.0","id":3,"method":"tools/list"})).unwrap();
        let t = &r["result"]["tools"][0];
        assert_eq!(t["name"], "convert_text");
        assert_eq!(t["annotations"]["readOnlyHint"], true);

        let r = handle(&json!({"jsonrpc":"2.0","id":4,"method":"tools/call",
            "params":{"name":"convert_text","arguments":{"text":"O'zbekiston shaharlari"}}}))
        .unwrap();
        let out = r["result"]["structuredContent"]["text"].as_str().unwrap();
        assert_eq!(out, convert("O'zbekiston shaharlari", &Options::default()));
        assert!(out.starts_with('Ö'), "{out}");
        assert_eq!(r["result"]["isError"], false);

        let r = handle(&json!({"jsonrpc":"2.0","id":5,"method":"tools/call",
            "params":{"name":"convert_text","arguments":{"text":"Özbekiston","direction":"to_cyrillic"}}}))
        .unwrap();
        assert_eq!(r["result"]["structuredContent"]["text"], "Ўзбекистон");
    }

    fn meta(v: &str) -> Value {
        json!({"io.modelcontextprotocol/protocolVersion": v, "io.modelcontextprotocol/clientCapabilities": {}})
    }

    #[test]
    fn modern_discover_and_calls() {
        let (c, r) = handle_http(&json!({"jsonrpc":"2.0","id":"d","method":"server/discover",
            "params":{"_meta": meta(MODERN)}}))
        .unwrap();
        assert_eq!(c, 200);
        let res = &r["result"];
        assert_eq!(res["resultType"], "complete");
        assert_eq!(res["supportedVersions"][0], MODERN);
        assert_eq!(
            res["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
            "bitg"
        );
        assert_eq!(res["cacheScope"], "public");

        let (c, r) = handle_http(&json!({"jsonrpc":"2.0","id":2,"method":"tools/list",
            "params":{"_meta": meta(MODERN)}}))
        .unwrap();
        assert_eq!(c, 200);
        assert_eq!(r["result"]["resultType"], "complete");
        assert_eq!(r["result"]["ttlMs"], TTL_MS);
        assert_eq!(r["result"]["tools"][0]["name"], "convert_text");

        let (c, r) = handle_http(&json!({"jsonrpc":"2.0","id":3,"method":"tools/call",
            "params":{"name":"convert_text","arguments":{"text":"shahar"},"_meta": meta(MODERN)}}))
        .unwrap();
        assert_eq!(c, 200);
        assert_eq!(r["result"]["resultType"], "complete");
        assert_eq!(r["result"]["structuredContent"]["text"], "şahar");

        let (c, r) = handle_http(&json!({"jsonrpc":"2.0","id":4,"method":"nope",
            "params":{"_meta": meta(MODERN)}}))
        .unwrap();
        assert_eq!((c, r["error"]["code"].as_i64()), (404, Some(-32601)));

        let (c, r) = handle_http(&json!({"jsonrpc":"2.0","id":5,"method":"tools/list",
            "params":{"_meta": meta("1900-01-01")}}))
        .unwrap();
        assert_eq!((c, r["error"]["code"].as_i64()), (400, Some(-32022)));
        assert_eq!(r["error"]["data"]["supported"][0], MODERN);
    }

    #[test]
    fn legacy_stays_200_without_result_type() {
        let (c, r) = handle_http(&json!({"jsonrpc":"2.0","id":1,"method":"tools/list"})).unwrap();
        assert_eq!(c, 200);
        assert!(r["result"].get("resultType").is_none());
        let (c, r) = handle_http(&json!({"jsonrpc":"2.0","id":2,"method":"x/y"})).unwrap();
        assert_eq!((c, r["error"]["code"].as_i64()), (200, Some(-32601)));
        assert!(
            handle_http(&json!({"jsonrpc":"2.0","method":"notifications/initialized"})).is_none()
        );
    }

    #[test]
    fn bad_input_is_tool_error() {
        let long = "a".repeat(MAX_TEXT + 1);
        let r = handle(&json!({"jsonrpc":"2.0","id":6,"method":"tools/call",
            "params":{"name":"convert_text","arguments":{"text": long}}}))
        .unwrap();
        assert_eq!(r["result"]["isError"], true);
        let r = handle(&json!({"jsonrpc":"2.0","id":7,"method":"tools/call",
            "params":{"name":"convert_text","arguments":{}}}))
        .unwrap();
        assert_eq!(r["result"]["isError"], true);
        let r = handle(&json!({"jsonrpc":"2.0","id":8,"method":"tools/call",
            "params":{"name":"nope","arguments":{}}}))
        .unwrap();
        assert_eq!(r["error"]["code"], -32602);
        let r = handle(&json!({"jsonrpc":"2.0","id":9,"method":"x/y"})).unwrap();
        assert_eq!(r["error"]["code"], -32601);
    }
}
