# Linux dvigatellarini lokal yig'ish va sinash muhiti (Ubuntu 24.04, CI bilan bir xil paketlar).
#   docker build -t bitg-linux -f linux/Dockerfile linux
#   docker run --rm -v "$PWD:/src" -w /src bitg-linux linux/ci.sh
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential pkg-config curl ca-certificates git \
      libibus-1.0-dev ibus dbus dbus-x11 python3 python3-gi gir1.2-ibus-1.0 \
      fcitx5 libfcitx5core-dev libfcitx5config-dev libfcitx5utils-dev fcitx5-modules-dev cmake extra-cmake-modules \
      dpkg-dev fakeroot lintian \
    && rm -rf /var/lib/apt/lists/*
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
ENV PATH=/root/.cargo/bin:$PATH
