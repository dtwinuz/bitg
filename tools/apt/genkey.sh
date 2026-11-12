#!/usr/bin/env bash
# bitg apt ombori imzo kalitini BIR MARTA yaratadi (deploy qiladigan kompyuterda).
#   maxfiy kalit:  ${BITG_APT_GNUPG:-~/.bitg-apt-gnupg}  (repoga KIRMAYDI; zaxira nusxasini OpenBao'ga qo'ying)
#   ochiq kalit:   deploy/apt/bitg.asc, barmoq izi: deploy/apt/fingerprint.txt (repoda)
# Kalit 3 yil amal qiladi. Uzaytirish: gpg --homedir "$H" --quick-set-expire <fpr> 3y, so'ng bitg.asc ni qayta eksport qiling.
set -euo pipefail
cd "$(dirname "$0")/../.."
H="${BITG_APT_GNUPG:-$HOME/.bitg-apt-gnupg}"
[ -f deploy/apt/fingerprint.txt ] && { echo "kalit allaqachon bor: $(cat deploy/apt/fingerprint.txt)"; exit 1; }
mkdir -p "$H" deploy/apt && chmod 700 "$H"
export GNUPGHOME="$H"
gpg --batch --passphrase '' --quick-gen-key "bitg apt repository <apt@bitg.uz>" rsa4096 sign 3y
FPR="$(gpg --with-colons --list-secret-keys | awk -F: '/^fpr:/ {print $10; exit}')"
gpg --armor --export "$FPR" > deploy/apt/bitg.asc
printf '%s\n' "$FPR" > deploy/apt/fingerprint.txt
echo "tayyor: $FPR"
echo "zaxira: gpg --homedir \"$H\" --armor --export-secret-keys $FPR  (OpenBao'ga, faylga yozib qoldirmang)"
