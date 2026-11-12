/* bitg yozish dvigateli — C ABI (crates/bitg-ime, statik kutubxona libbitg_ime.a).
 * Barcha satrlar UTF-8. Qaytgan ko'rsatkichlar dvigatelga tegishli va shu dvigatelning keyingi chaqiruvigacha
 * yaroqli: bo'shatilmaydi, saqlab qo'yilmaydi. Bitta dvigatel — bitta kiritish konteksti, bitta oqim. */
#ifndef BITG_IME_H
#define BITG_IME_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct BitgIme BitgIme;

enum {
    BITG_KEY_CHAR = 0,      /* codepoint — bosilgan belgi (tartib qo'llangan) */
    BITG_KEY_SPACE = 1,     /* codepoint — ' ' yoki U+00A0 */
    BITG_KEY_ENTER = 2,
    BITG_KEY_TAB = 3,
    BITG_KEY_BACKSPACE = 4,
    BITG_KEY_NAVIGATE = 5,  /* strelkalar, Home, End, PgUp, PgDn, Delete, Esc */
    BITG_KEY_OTHER = 6      /* Ctrl/Alt/Super bilan yoki belgisiz tugma */
};

/* Qo'llash tartibi: delete_before -> commit -> preedit; consumed == 0 bo'lsa tugma dasturga o'tadi. */
typedef struct {
    uint32_t consumed;
    uint32_t delete_before;   /* kursor oldidan o'chiriladigan belgilar (Unicode scalar) */
    const char *commit;       /* yoziladigan matn yoki NULL */
    uint32_t preedit_changed; /* 1 — preedit `preedit` bilan almashtirilsin ("" — yashirilsin) */
    const char *preedit;
} BitgImeOut;

/* Sozlamalar (settings.ini, exceptions.txt): Linux — $XDG_CONFIG_HOME/bitg/, macOS — ~/Library/Application Support/bitg/.
 * Xato bo'lsa NULL. */
BitgIme *bitg_ime_new(void);
void bitg_ime_free(BitgIme *ime);

/* Faqat tugma bosilishi (qo'yib yuborish emas) va faqat modifikator bo'lmagan tugmalar.
 * before — kursor oldidagi matn (dastur surrounding text bersa va tanlov bo'lmasa) yoki NULL. */
const BitgImeOut *bitg_ime_key(BitgIme *ime, uint32_t kind, uint32_t codepoint, const char *before);

void bitg_ime_focus_in(BitgIme *ime);
/* Fokus ketdi yoki kursor siljidi. Preedit'dagi o'girilmagan so'z (yoki NULL): dastur preedit'ni
 * o'zi yozmasa, frontend uni yozadi. */
const char *bitg_ime_reset(BitgIme *ime);
/* Parol, PIN, terminal maydoni: on != 0 — dvigatel hech narsaga tegmaydi. */
const BitgImeOut *bitg_ime_set_suspended(BitgIme *ime, uint32_t on);

#ifdef __cplusplus
}
#endif

#endif
