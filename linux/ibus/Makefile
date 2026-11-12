# ibus-engine-bitg: avval `cargo build --release -p bitg-ime` (libbitg_ime.a), keyin `make -C linux/ibus`.
CARGO_TARGET ?= ../../target
LIB ?= $(CARGO_TARGET)/release/libbitg_ime.a
VERSION ?= 0.0.0
CFLAGS ?= -O2 -g
WARN := -Wall -Wextra -Wno-unused-parameter
# Rust staticlib talab qiladigan tizim kutubxonalari (`rustc --print native-static-libs`)
NATIVE_LIBS ?= -lgcc_s -lutil -lrt -lpthread -lm -ldl -lc

IBUS_CFLAGS := $(shell pkg-config --cflags ibus-1.0)
IBUS_LIBS := $(shell pkg-config --libs ibus-1.0)

ibus-engine-bitg: engine.c ../include/bitg_ime.h $(LIB)
	$(CC) $(CFLAGS) $(WARN) $(CPPFLAGS) -DBITG_VERSION='"$(VERSION)"' -I../include $(IBUS_CFLAGS) \
		engine.c -o $@ $(LDFLAGS) $(LIB) $(IBUS_LIBS) $(NATIVE_LIBS)

clean:
	rm -f ibus-engine-bitg

.PHONY: clean
