# bitg integrations for AI assistants

Public MCP endpoint: `https://bitg.uz/mcp` (Streamable HTTP, no login, one read-only tool `convert_text`).
Text is processed in memory and not stored or logged; see https://bitg.uz/maxfiylik/.

| Client | How to add |
|---|---|
| Claude Code | `claude plugin marketplace add dtwinuz/bitg` then `claude plugin install bitg@bitg` (or `claude mcp add --transport http bitg https://bitg.uz/mcp`) |
| Claude.ai / Desktop | Settings → Connectors → Add custom connector → `https://bitg.uz/mcp` |
| ChatGPT | Settings → Apps → developer mode → Create → `https://bitg.uz/mcp` (directory listing pending review) |
| Gemini CLI | `gemini extensions install https://github.com/dtwinuz/bitg` |
| VS Code / Cursor / others | add an HTTP MCP server with URL `https://bitg.uz/mcp` |
| MCP Registry | `uz.bitg/bitg` (`server.json`) |

Files: `../server.json` (MCP Registry), `../gemini-extension.json` + `../GEMINI.md` (Gemini CLI), `../.claude-plugin/marketplace.json` + `claude/` (Claude Code plugin with skill).

Not an AI integration, but in the same folder: `google-workspace/` is a Google Docs, Sheets and Slides editor add-on that runs the bitg engine (wasm) inside Apps Script, so no text is sent to bitg.uz. See `google-workspace/README.md`.
