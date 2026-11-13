// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Fcitx5 qo'shimchasi (bitg.so). Mantiq — crates/bitg-ime (libbitg_ime.a); bu fayl faqat Fcitx5 bilan bog'laydi.
// Har kiritish kontekstiga alohida dvigatel (InputContextProperty).
#include <fcitx-utils/key.h>
#include <fcitx-utils/utf8.h>
#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputcontextproperty.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputpanel.h>
#include <fcitx/instance.h>
#include <fcitx/text.h>

#include <optional>
#include <string>

#include "bitg_ime.h"

namespace {

class BitgState : public fcitx::InputContextProperty {
public:
    BitgState() : ime(bitg_ime_new()) {}
    ~BitgState() override { bitg_ime_free(ime); }
    BitgState(const BitgState &) = delete;
    BitgState &operator=(const BitgState &) = delete;
    BitgIme *ime;
};

bool suspendedFor(const fcitx::InputContext *ic) {
    const auto caps = ic->capabilityFlags();
    return caps.test(fcitx::CapabilityFlag::Password) || caps.test(fcitx::CapabilityFlag::Sensitive) ||
           caps.test(fcitx::CapabilityFlag::Email) || caps.test(fcitx::CapabilityFlag::Terminal);
}

// Kursor oldidan olinadigan belgilar soni. So'z ko'pi bilan 40 belgi; macOS va IBus ham 128 oladi.
constexpr size_t kBeforeWindow = 128;

// Kursor oldidagi oxirgi kBeforeWindow belgi: dastur bermasa yoki tanlov bo'lsa — yo'q.
std::optional<std::string> textBeforeCursor(fcitx::InputContext *ic) {
    if (!ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText)) {
        return std::nullopt;
    }
    const auto &s = ic->surroundingText();
    if (!s.isValid() || s.cursor() != s.anchor()) {
        return std::nullopt;
    }
    const std::string &text = s.text();
    const auto len = fcitx::utf8::lengthValidated(text);
    if (len == fcitx::utf8::INVALID_LENGTH || s.cursor() > len) {
        return std::nullopt;
    }
    const size_t skip = s.cursor() > kBeforeWindow ? s.cursor() - kBeforeWindow : 0;
    auto start = fcitx::utf8::nextNChar(text.begin(), skip);
    auto end = fcitx::utf8::nextNChar(start, s.cursor() - skip);
    return std::string(start, end);
}

uint32_t keyKind(const fcitx::Key &key, uint32_t &cp) {
    using fcitx::KeyState;
    cp = 0;
    if (key.states().testAny(fcitx::KeyStates{KeyState::Ctrl, KeyState::Alt, KeyState::Super, KeyState::Hyper,
                                              KeyState::Meta})) {
        return BITG_KEY_OTHER;
    }
    switch (key.sym()) {
    case FcitxKey_space:
    case FcitxKey_KP_Space:
        cp = ' ';
        return BITG_KEY_SPACE;
    case FcitxKey_nobreakspace:
        cp = 0xA0;
        return BITG_KEY_SPACE;
    case FcitxKey_Return:
    case FcitxKey_KP_Enter:
    case FcitxKey_ISO_Enter:
        return BITG_KEY_ENTER;
    case FcitxKey_Tab:
    case FcitxKey_KP_Tab:
    case FcitxKey_ISO_Left_Tab:
        return BITG_KEY_TAB;
    case FcitxKey_BackSpace:
        return BITG_KEY_BACKSPACE;
    case FcitxKey_Left: case FcitxKey_Right: case FcitxKey_Up: case FcitxKey_Down:
    case FcitxKey_Home: case FcitxKey_End: case FcitxKey_Page_Up: case FcitxKey_Page_Down:
    case FcitxKey_KP_Left: case FcitxKey_KP_Right: case FcitxKey_KP_Up: case FcitxKey_KP_Down:
    case FcitxKey_KP_Home: case FcitxKey_KP_End: case FcitxKey_KP_Page_Up: case FcitxKey_KP_Page_Down:
    case FcitxKey_Delete: case FcitxKey_KP_Delete: case FcitxKey_Escape:
        return BITG_KEY_NAVIGATE;
    default:
        break;
    }
    const uint32_t u = fcitx::Key::keySymToUnicode(key.sym());
    if (u >= 0x20 && u != 0x7F && !(u >= 0x80 && u < 0xA0)) {
        cp = u;
        return BITG_KEY_CHAR;
    }
    return BITG_KEY_OTHER;
}

class BitgEngine : public fcitx::InputMethodEngineV2 {
public:
    explicit BitgEngine(fcitx::Instance *instance)
        : instance_(instance), factory_([](fcitx::InputContext &) { return new BitgState; }) {
        instance_->inputContextManager().registerProperty("bitgState", &factory_);
    }

    void keyEvent(const fcitx::InputMethodEntry &, fcitx::KeyEvent &event) override {
        if (event.isRelease() || event.key().isModifier()) {
            return;
        }
        auto *ic = event.inputContext();
        auto *st = ic->propertyFor(&factory_);
        if (st->ime == nullptr) {
            return;
        }
        apply(ic, bitg_ime_set_suspended(st->ime, suspendedFor(ic)));
        uint32_t cp = 0;
        const uint32_t kind = keyKind(event.key(), cp);
        // matn faqat dvigatelga kerak bo'lganda so'raladi: katta hujjatda har tugmada nusxa olinmaydi
        const auto before =
            bitg_ime_wants_before(st->ime, kind, cp) ? textBeforeCursor(ic) : std::optional<std::string>{};
        const BitgImeOut *o = bitg_ime_key(st->ime, kind, cp, before ? before->c_str() : nullptr);
        apply(ic, o);
        if (o != nullptr && o->consumed) {
            event.filterAndAccept();
        }
    }

    void activate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) override {
        auto *st = event.inputContext()->propertyFor(&factory_);
        bitg_ime_focus_in(st->ime);
    }

    void deactivate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override {
        reset(entry, event);
    }

    // Fokus ketdi, kursor siljidi: preedit'dagi so'z o'girilmay yoziladi (matn yo'qolmasin).
    void reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) override {
        auto *ic = event.inputContext();
        auto *st = ic->propertyFor(&factory_);
        if (const char *pending = bitg_ime_reset(st->ime)) {
            ic->commitString(pending);
        }
        ic->inputPanel().setClientPreedit(fcitx::Text());
        ic->updatePreedit();
    }

private:
    static void apply(fcitx::InputContext *ic, const BitgImeOut *o) {
        if (o == nullptr) {
            return;
        }
        if (o->delete_before > 0) {
            ic->deleteSurroundingText(-static_cast<int>(o->delete_before), o->delete_before);
        }
        if (o->commit != nullptr) {
            ic->commitString(o->commit);
        }
        if (o->preedit_changed) {
            fcitx::Text t;
            if (o->preedit != nullptr && o->preedit[0] != '\0') {
                t.append(o->preedit, fcitx::TextFormatFlag::Underline);
                t.setCursor(static_cast<int>(std::string(o->preedit).size()));
            }
            ic->inputPanel().setClientPreedit(t);
            ic->updatePreedit();
        }
    }

    fcitx::Instance *instance_;
    fcitx::FactoryFor<BitgState> factory_;
};

class BitgFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *manager) override { return new BitgEngine(manager->instance()); }
};

} // namespace

FCITX_ADDON_FACTORY(BitgFactory);
