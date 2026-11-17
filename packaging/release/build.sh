#!/usr/bin/env bash
set -euo pipefail

name='mime-guardian'
bin='mmg'

die() {
	echo "release: $1" >&2
	exit 1
}

[ $# -eq 1 ] || die "usage: $0 <tag>"

tag=$1
[ -n "$tag" ] || die 'no tag given; nothing was released'

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$root"

pkgver=${tag#v}
pkgver=${pkgver//-/.}

declare -A rust_targets=(
	[x86_64]=x86_64-unknown-linux-gnu
	[aarch64]=aarch64-unknown-linux-gnu
)

declare -A completion_files=(
	[bash]=$bin.bash
	[zsh]=_$bin
	[fish]=$bin.fish
)

host=$(uname -m)
[ -n "${rust_targets[$host]+set}" ] || die "$host is not a released architecture"

export RUSTFLAGS="${RUSTFLAGS:-} -C strip=symbols"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

dist="$root/dist"
rm -rf "$dist"
mkdir -p "$dist"

for arch in "${!rust_targets[@]}"; do
	echo "release: building $pkgver for $arch"
	cargo build --release --locked --target "${rust_targets[$arch]}"
done

host_bin="target/${rust_targets[$host]}/release/$bin"
completions="$dist/completions"
mkdir -p "$completions"
for shell in "${!completion_files[@]}"; do
	"$host_bin" completions "$shell" >"$completions/${completion_files[$shell]}"
done

manual="$dist/$bin.1"
"$host_bin" man >"$manual"

for arch in "${!rust_targets[@]}"; do
	stage="$dist/$name-$pkgver-$arch"
	install -Dm0755 "target/${rust_targets[$arch]}/release/$bin" "$stage/$bin"
	install -Dm0644 LICENSE "$stage/LICENSE"
	install -Dm0644 "contrib/$name.service" "$stage/$name.service"
	install -Dm0644 "contrib/$name.desktop" "$stage/$name.desktop"
	install -Dm0644 "contrib/$name.svg" "$stage/$name.svg"
	install -Dm0644 "$manual" "$stage/$bin.1"
	install -d "$stage/completions"
	install -m0644 "$completions"/* "$stage/completions/"
	tar -C "$dist" -czf "$dist/$name-$pkgver-$arch.tar.gz" "$name-$pkgver-$arch"
	rm -rf "$stage"
	echo "release: packed $name-$pkgver-$arch.tar.gz"
done

rm -rf "$completions" "$manual"
