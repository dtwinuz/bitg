# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

# Linux dvigatellarini lokal yig'ish va sinash muhiti (Ubuntu 24.04, CI bilan bir xil paketlar).
#   docker build -t bitg-linux -f linux/Dockerfile linux
#   docker run --rm -v "$PWD:/src" -w /src bitg-linux linux/ci.sh
FROM ubuntu:25.10@sha256:7cc5e35f6567ee8c66d2abb4aab0fd866669e6207c237c3a8f0947a5c7f17092
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential pkg-config curl ca-certificates git \
      libibus-1.0-dev ibus dbus dbus-x11 python3 python3-gi gir1.2-ibus-1.0 \
      fcitx5 libfcitx5core-dev libfcitx5config-dev libfcitx5utils-dev fcitx5-modules-dev cmake extra-cmake-modules \
      dpkg-dev fakeroot lintian \
    && rm -rf /var/lib/apt/lists/*
# rustup-init versiyasi va SHA-256 qotirilgan (static.rust-lang.org/rustup/archive/<v>/.../rustup-init.sha256)
ARG RUSTUP_VERSION=1.29.2
ARG RUSTUP_SHA256=5be191e8c70a5007074254cb74d78dbfdd25cd5a61c27de7b8e5410122ec24f4
RUN curl -sSfLo /tmp/rustup-init "https://static.rust-lang.org/rustup/archive/${RUSTUP_VERSION}/x86_64-unknown-linux-gnu/rustup-init"     && echo "${RUSTUP_SHA256}  /tmp/rustup-init" | sha256sum -c -     && chmod +x /tmp/rustup-init && /tmp/rustup-init -y --profile minimal --default-toolchain stable && rm /tmp/rustup-init
ENV PATH=/root/.cargo/bin:$PATH
