#!/usr/bin/env bash
set -euo pipefail

name='mime-guardian'
bin='mmg'
maintainer='ItzTas <ts.aur@imts.aleeas.com>'
homepage='https://github.com/ItzTas/mime-guardian'
revision=1

die() {
	echo "release: $1" >&2
	exit 1
}

[ $# -eq 1 ] || die "usage: $0 <tag>"

tag=$1
[ -n "$tag" ] || die 'no tag given; nothing was released'

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
dist="$root/dist"
[ -d "$dist" ] || die 'no binaries to package; run build.sh first'

pkgver=${tag#v}
pkgver=${pkgver//-/.}
version=${tag#v}
version=${version//-/\~}

declare -A debian_archs=(
	["x86_64"]=amd64
	["aarch64"]=arm64
)

glibc_floor() {
	readelf --wide --dyn-syms "$1" |
		grep -o 'GLIBC_[0-9]\+\.[0-9]\+' |
		sort -u -V |
		tail -1 |
		cut -d _ -f 2
}

for arch in "${!debian_archs[@]}"; do
	tarball="$dist/$name-$pkgver-$arch.tar.gz"
	[ -f "$tarball" ] || die "missing $(basename "$tarball"); run build.sh first"

	debian_arch=${debian_archs[$arch]}
	source="$dist/$name-$pkgver-$arch"
	tree="$dist/deb-$debian_arch"
	rm -rf "$source" "$tree"
	tar -C "$dist" -xzf "$tarball"

	install -Dm0755 "$source/$bin" "$tree/usr/bin/$bin"
	install -Dm0644 "$source/LICENSE" "$tree/usr/share/doc/$name/copyright"
	install -Dm0644 "$source/$name.service" "$tree/usr/lib/systemd/user/$name.service"
	install -Dm0644 "$source/completions/$bin.bash" \
		"$tree/usr/share/bash-completion/completions/$bin"
	install -Dm0644 "$source/completions/_$bin" \
		"$tree/usr/share/zsh/vendor-completions/_$bin"
	install -Dm0644 "$source/completions/$bin.fish" \
		"$tree/usr/share/fish/vendor_completions.d/$bin.fish"
	install -Dm0644 "$source/$bin.1" "$tree/usr/share/man/man1/$bin.1"
	gzip -9n "$tree/usr/share/man/man1/$bin.1"

	installed_size=$(du -ks "$tree" | cut -f 1)
	glibc=$(glibc_floor "$source/$bin")
	rm -rf "$source"

	install -d "$tree/DEBIAN"
	cat >"$tree/DEBIAN/control" <<-EOF
		Package: $name
		Version: $version-$revision
		Architecture: $debian_arch
		Maintainer: $maintainer
		Installed-Size: $installed_size
		Depends: libc6 (>= $glibc), libgcc-s1 (>= 3.0)
		Recommends: shared-mime-info, libnotify-bin
		Section: utils
		Priority: optional
		Homepage: $homepage
		Description: Keeps your default applications the way you set them
		 mmg watches the mimeapps.list files that decide which application opens
		 each file type and link, and puts your choice back when a program changes it.
	EOF

	(cd "$tree" && find . -type f -not -path './DEBIAN/*' -printf '%P\n' |
		sort |
		xargs -r md5sum >DEBIAN/md5sums)

	deb="$dist/${name}_$version-${revision}_$debian_arch.deb"
	dpkg-deb --build --root-owner-group "$tree" "$deb" >/dev/null
	rm -rf "$tree"
	echo "release: packed $(basename "$deb")"
done
