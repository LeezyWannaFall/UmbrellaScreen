#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
version=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' Cargo.toml | head -n 1)
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'Expected a numeric Cargo version' >&2; exit 1; }
output=$(realpath -m "${1:-target/packages}")
mkdir -p "$output"
sed "s/@VERSION@/$version/g" packaging/PKGBUILD.in > "$output/PKGBUILD"
# Export tracked sources only, excluding local libraries and Cargo build outputs.
git archive --format=tar --prefix="umbrellascreen-$version/" HEAD | gzip -n > "$output/umbrellascreen-$version.tar.gz"
cd "$output"
makepkg --geninteg >> PKGBUILD
makepkg --force --noconfirm
makepkg --printsrcinfo > .SRCINFO
sha256sum umbrellascreen-*.pkg.tar.zst > SHA256SUMS
