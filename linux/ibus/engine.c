/* ibus-engine-bitg: IBus dvigateli. Mantiq — crates/bitg-ime (libbitg_ime.a), bu fayl faqat IBus bilan bog'laydi.
 *
 *   ibus-engine-bitg --ibus     ibus-daemon ishga tushiradi (component/bitg.xml)
 *   ibus-engine-bitg            qo'lda: komponentni o'zi ro'yxatdan o'tkazadi (sinov uchun)
 */
#include <ibus.h>
#include <string.h>

#include "bitg_ime.h"

#ifndef BITG_VERSION
#define BITG_VERSION "0.0.0"
#endif

/* G_DECLARE_FINAL_TYPE emas: IBus 1.5.26 (Ubuntu 22.04) IBusEngine uchun g_autoptr e'lon qilmaydi */
typedef struct {
    IBusEngine parent;
    BitgIme *ime;
} BitgEngine;

typedef struct {
    IBusEngineClass parent;
} BitgEngineClass;

GType bitg_engine_get_type(void);
#define BITG_TYPE_ENGINE (bitg_engine_get_type())
#define BITG_ENGINE(o) (G_TYPE_CHECK_INSTANCE_CAST((o), BITG_TYPE_ENGINE, BitgEngine))

G_DEFINE_TYPE(BitgEngine, bitg_engine, IBUS_TYPE_ENGINE)

static gboolean has_surrounding(IBusEngine *e)
{
    return (e->client_capabilities & IBUS_CAP_SURROUNDING_TEXT) != 0;
}

/* Kursor oldidagi matn (yangi satr, chaqiruvchi g_free qiladi) yoki NULL: dastur bermasa yoki tanlov bo'lsa. */
static gchar *text_before_cursor(IBusEngine *e)
{
    if (!has_surrounding(e))
        return NULL;
    IBusText *text = NULL;
    guint cursor = 0, anchor = 0;
    ibus_engine_get_surrounding_text(e, &text, &cursor, &anchor);
    if (text == NULL)
        return NULL;
    gchar *out = NULL;
    if (cursor == anchor) {
        const gchar *s = ibus_text_get_text(text);
        glong n = g_utf8_strlen(s, -1);
        if ((glong)cursor <= n)
            out = g_strndup(s, g_utf8_offset_to_pointer(s, cursor) - s);
    }
    g_object_unref(text);
    return out;
}

static void apply(IBusEngine *e, const BitgImeOut *o)
{
    if (o == NULL)
        return;
    if (o->delete_before > 0)
        ibus_engine_delete_surrounding_text(e, -(gint)o->delete_before, o->delete_before);
    if (o->commit != NULL)
        ibus_engine_commit_text(e, ibus_text_new_from_string(o->commit));
    if (o->preedit_changed) {
        if (o->preedit == NULL || o->preedit[0] == '\0') {
            ibus_engine_update_preedit_text_with_mode(e, ibus_text_new_from_static_string(""), 0, FALSE,
                                                      IBUS_ENGINE_PREEDIT_CLEAR);
            ibus_engine_hide_preedit_text(e);
        } else {
            IBusText *t = ibus_text_new_from_string(o->preedit);
            guint len = ibus_text_get_length(t);
            ibus_text_append_attribute(t, IBUS_ATTR_TYPE_UNDERLINE, IBUS_ATTR_UNDERLINE_SINGLE, 0, len);
            /* COMMIT: fokus ketsa yoki sichqoncha bosilsa IBus so'zni o'zi yozadi — matn yo'qolmaydi */
            ibus_engine_update_preedit_text_with_mode(e, t, len, TRUE, IBUS_ENGINE_PREEDIT_COMMIT);
        }
    }
}

static gboolean is_modifier(guint keyval)
{
    switch (keyval) {
    case IBUS_KEY_Shift_L: case IBUS_KEY_Shift_R: case IBUS_KEY_Control_L: case IBUS_KEY_Control_R:
    case IBUS_KEY_Alt_L: case IBUS_KEY_Alt_R: case IBUS_KEY_Meta_L: case IBUS_KEY_Meta_R:
    case IBUS_KEY_Super_L: case IBUS_KEY_Super_R: case IBUS_KEY_Hyper_L: case IBUS_KEY_Hyper_R:
    case IBUS_KEY_Caps_Lock: case IBUS_KEY_Shift_Lock: case IBUS_KEY_Num_Lock: case IBUS_KEY_Scroll_Lock:
    case IBUS_KEY_ISO_Level3_Shift: case IBUS_KEY_ISO_Level5_Shift: case IBUS_KEY_Mode_switch:
    case IBUS_KEY_ISO_Next_Group: case IBUS_KEY_ISO_Prev_Group:
        return TRUE;
    default:
        return FALSE;
    }
}

static guint key_kind(guint keyval, guint state, guint32 *cp)
{
    *cp = 0;
    if (state & (IBUS_CONTROL_MASK | IBUS_MOD1_MASK | IBUS_SUPER_MASK | IBUS_HYPER_MASK | IBUS_META_MASK))
        return BITG_KEY_OTHER;
    switch (keyval) {
    case IBUS_KEY_space: case IBUS_KEY_KP_Space:
        *cp = ' ';
        return BITG_KEY_SPACE;
    case IBUS_KEY_nobreakspace:
        *cp = 0xA0;
        return BITG_KEY_SPACE;
    case IBUS_KEY_Return: case IBUS_KEY_KP_Enter: case IBUS_KEY_ISO_Enter:
        return BITG_KEY_ENTER;
    case IBUS_KEY_Tab: case IBUS_KEY_KP_Tab: case IBUS_KEY_ISO_Left_Tab:
        return BITG_KEY_TAB;
    case IBUS_KEY_BackSpace:
        return BITG_KEY_BACKSPACE;
    case IBUS_KEY_Left: case IBUS_KEY_Right: case IBUS_KEY_Up: case IBUS_KEY_Down:
    case IBUS_KEY_Home: case IBUS_KEY_End: case IBUS_KEY_Page_Up: case IBUS_KEY_Page_Down:
    case IBUS_KEY_KP_Left: case IBUS_KEY_KP_Right: case IBUS_KEY_KP_Up: case IBUS_KEY_KP_Down:
    case IBUS_KEY_KP_Home: case IBUS_KEY_KP_End: case IBUS_KEY_KP_Page_Up: case IBUS_KEY_KP_Page_Down:
    case IBUS_KEY_Delete: case IBUS_KEY_KP_Delete: case IBUS_KEY_Escape:
        return BITG_KEY_NAVIGATE;
    default:
        break;
    }
    gunichar u = ibus_keyval_to_unicode(keyval);
    if (u >= 0x20 && u != 0x7F && !g_unichar_iscntrl(u)) {
        *cp = u;
        return BITG_KEY_CHAR;
    }
    return BITG_KEY_OTHER;
}

static gboolean bitg_engine_process_key_event(IBusEngine *e, guint keyval, guint keycode, guint state)
{
    (void)keycode;
    BitgEngine *self = BITG_ENGINE(e);
    if (self->ime == NULL || (state & IBUS_RELEASE_MASK) || is_modifier(keyval))
        return FALSE;
    guint32 cp;
    guint kind = key_kind(keyval, state, &cp);
    gchar *before = text_before_cursor(e);
    const BitgImeOut *o = bitg_ime_key(self->ime, kind, cp, before);
    g_free(before);
    apply(e, o);
    return o != NULL && o->consumed;
}

static void bitg_engine_focus_in(IBusEngine *e)
{
    BitgEngine *self = BITG_ENGINE(e);
    bitg_ime_focus_in(self->ime);
    /* surrounding text'ni oldindan so'raymiz: IBus uni birinchi so'rovdan keyin yubora boshlaydi */
    if (has_surrounding(e)) {
        gchar *b = text_before_cursor(e);
        g_free(b);
    }
    IBUS_ENGINE_CLASS(bitg_engine_parent_class)->focus_in(e);
}

/* Preedit'dagi xom so'zni ibus-daemon o'zi yozadi (PREEDIT_COMMIT), bu yerda faqat holat tozalanadi. */
static void forget(IBusEngine *e)
{
    BitgEngine *self = BITG_ENGINE(e);
    (void)bitg_ime_reset(self->ime);
}

static void bitg_engine_focus_out(IBusEngine *e)
{
    forget(e);
    IBUS_ENGINE_CLASS(bitg_engine_parent_class)->focus_out(e);
}

static void bitg_engine_reset(IBusEngine *e)
{
    forget(e);
    IBUS_ENGINE_CLASS(bitg_engine_parent_class)->reset(e);
}

static void bitg_engine_disable(IBusEngine *e)
{
    forget(e);
    IBUS_ENGINE_CLASS(bitg_engine_parent_class)->disable(e);
}

static void bitg_engine_set_content_type(IBusEngine *e, guint purpose, guint hints)
{
    BitgEngine *self = BITG_ENGINE(e);
    gboolean off = purpose == IBUS_INPUT_PURPOSE_PASSWORD || purpose == IBUS_INPUT_PURPOSE_PIN ||
                   purpose == IBUS_INPUT_PURPOSE_TERMINAL || purpose == IBUS_INPUT_PURPOSE_EMAIL;
    apply(e, bitg_ime_set_suspended(self->ime, off));
    IBUS_ENGINE_CLASS(bitg_engine_parent_class)->set_content_type(e, purpose, hints);
}

static void bitg_engine_init(BitgEngine *self)
{
    self->ime = bitg_ime_new();
    if (self->ime == NULL)
        g_warning("bitg: dvigatel yaratilmadi, tugmalar o'zgarishsiz o'tadi");
}

static void bitg_engine_dispose(GObject *obj)
{
    BitgEngine *self = BITG_ENGINE(obj);
    bitg_ime_free(self->ime);
    self->ime = NULL;
    G_OBJECT_CLASS(bitg_engine_parent_class)->dispose(obj);
}

static void bitg_engine_class_init(BitgEngineClass *klass)
{
    G_OBJECT_CLASS(klass)->dispose = bitg_engine_dispose;
    IBusEngineClass *e = IBUS_ENGINE_CLASS(klass);
    e->process_key_event = bitg_engine_process_key_event;
    e->focus_in = bitg_engine_focus_in;
    e->focus_out = bitg_engine_focus_out;
    e->reset = bitg_engine_reset;
    e->disable = bitg_engine_disable;
    e->set_content_type = bitg_engine_set_content_type;
}

static IBusComponent *component(void)
{
    IBusComponent *c = ibus_component_new("org.freedesktop.IBus.Bitg", "bitg", BITG_VERSION, "MIT OR Apache-2.0",
                                          "dtwinuz", "https://bitg.uz", "", "bitg");
    ibus_component_add_engine(
        c, ibus_engine_desc_new("bitg", "bitg", "Yangi o'zbek alifbosi: yozganingiz so'z oxirida o'giriladi",
                                "uz", "MIT OR Apache-2.0", "dtwinuz", "bitg", "default"));
    return c;
}

static void quit(IBusBus *bus, gpointer data)
{
    (void)bus;
    (void)data;
    ibus_quit();
}

int main(int argc, char **argv)
{
    gboolean by_daemon = argc > 1 && strcmp(argv[1], "--ibus") == 0;
    ibus_init();
    IBusBus *bus = ibus_bus_new();
    if (!ibus_bus_is_connected(bus)) {
        g_printerr("bitg: ibus-daemon topilmadi\n");
        return 1;
    }
    g_signal_connect(bus, "disconnected", G_CALLBACK(quit), NULL);
    IBusFactory *factory = ibus_factory_new(ibus_bus_get_connection(bus));
    ibus_factory_add_engine(factory, "bitg", BITG_TYPE_ENGINE);
    if (by_daemon) {
        ibus_bus_request_name(bus, "org.freedesktop.IBus.Bitg", 0);
    } else {
        IBusComponent *c = component();
        ibus_bus_register_component(bus, c);
        g_object_unref(c);
    }
    ibus_main();
    return 0;
}
