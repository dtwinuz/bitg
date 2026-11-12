# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

# bitg.so (Fcitx5 qo'shimchasi): avval `cargo build --release -p bitg-ime`, keyin `make -C linux/fcitx5`.
CARGO_TARGET ?= ../../target
LIB ?= $(CARGO_TARGET)/release/libbitg_ime.a
CXXFLAGS ?= -O2 -g
WARN := -Wall -Wextra
NATIVE_LIBS ?= -lgcc_s -lutil -lrt -lpthread -lm -ldl -lc

FCITX_CFLAGS := $(shell pkg-config --cflags Fcitx5Core Fcitx5Utils)
FCITX_LIBS := $(shell pkg-config --libs Fcitx5Core Fcitx5Utils)

# -Wl,--exclude-libs,ALL: Rust belgilarini tashqariga chiqarmaymiz (boshqa qo'shimchalar bilan to'qnashmasin)
bitg.so: bitg.cpp ../include/bitg_ime.h $(LIB)
	$(CXX) -std=c++17 -fPIC -shared -fvisibility=hidden $(CXXFLAGS) $(WARN) $(CPPFLAGS) -I../include $(FCITX_CFLAGS) \
		bitg.cpp -o $@ $(LDFLAGS) -Wl,--exclude-libs,ALL -Wl,--no-undefined $(LIB) $(FCITX_LIBS) $(NATIVE_LIBS)

clean:
	rm -f bitg.so

.PHONY: clean
