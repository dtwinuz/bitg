#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Ubuntu input-source diagnostics for issue #75. Does not modify settings.
set -euo pipefail
printf 'bitg Ubuntu diagnostics (no typed text is collected)\n'
printf 'OS: '; ( . /etc/os-release && printf '%s\n' "${PRETTY_NAME:-unknown}" ) || true
printf 'Session: %s / %s\n' "${XDG_CURRENT_DESKTOP:-unknown}" "${XDG_SESSION_TYPE:-unknown}"
printf 'Keyboard layout: '; (setxkbmap -query 2>/dev/null | grep '^layout:' || true)
printf 'IBus: '; (ibus version 2>/dev/null | head -1 || true)
printf 'Fcitx5: '; (fcitx5 -v 2>/dev/null | head -1 || true)
printf 'Active IBus engine: '; (ibus engine 2>/dev/null || true)
printf 'bitg CLI: '; (bitg --version 2>/dev/null || true)
printf 'IBus bitg component: '; if test -f /usr/share/ibus/component/bitg.xml; then echo installed; else echo missing; fi
printf 'bitg configuration keys (values omitted):\n'
cfg="${XDG_CONFIG_HOME:-$HOME/.config}/bitg/settings.ini"
if test -f "$cfg"; then sed -nE 's/^[[:space:]]*([a-zA-Z_]+)[[:space:]]*=.*/  \1=<redacted>/p' "$cfg"; else echo '  defaults (no settings.ini)'; fi
printf '\nManual test (type in GTK text editor, then browser and Telegram):\n'
printf '  choy shahar <space> => çoy şahar <space>\n'
printf '  чой шаҳар <space> => çoy şahar <space>\n'
printf 'Repeat after switching input sources from Latin and Cyrillic layouts.\n'
printf 'Record app name, layout, before/after-space behavior, and errors.\n'
