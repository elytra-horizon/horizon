#!/usr/bin/env bash
# Packages the Linux launcher for Arch Linux (.pkg.tar.zst) and provides PKGBUILD for Elytra Horizon.
#
#   packaging/linux/arch.sh BINARY VERSION OUTDIR
#
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
binary=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
version=$2
out=$(mkdir -p "$3" && cd "$3" && pwd)

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

pkgroot=$work/pkg
mkdir -p "$pkgroot/usr/bin" \
         "$pkgroot/usr/share/applications" \
         "$pkgroot/usr/share/icons"

# Binaries: primary elytra-horizon and symlink horizon-launcher
cp "$binary" "$pkgroot/usr/bin/elytra-horizon"
chmod 755 "$pkgroot/usr/bin/elytra-horizon"
ln -s elytra-horizon "$pkgroot/usr/bin/horizon-launcher"

# Desktop entry
cat > "$pkgroot/usr/share/applications/elytra-horizon.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=Elytra Horizon
GenericName=Game launcher
Comment=Install, prepare and play Horizon (Minecraft: Java Edition in Rust)
Exec=elytra-horizon
Icon=elytra-horizon
Terminal=false
Categories=Game;
StartupWMClass=horizon-launcher
EOF
chmod 644 "$pkgroot/usr/share/applications/elytra-horizon.desktop"

# Icons
cp -R "$here/../icons/hicolor/." "$pkgroot/usr/share/icons/hicolor/"
find "$pkgroot/usr/share/icons" -name "horizon.png" | while read -r icon; do
  dir=$(dirname "$icon")
  cp "$icon" "$dir/elytra-horizon.png"
done
find "$pkgroot/usr/share/icons" -name "horizon.svg" | while read -r icon; do
  dir=$(dirname "$icon")
  cp "$icon" "$dir/elytra-horizon.svg"
done
find "$pkgroot/usr/share/icons" -type f -exec chmod 644 {} +

# Calculate installed size in bytes
size=$(du -sb "$pkgroot" | cut -f1)
builddate=$(date +%s)

cat > "$pkgroot/.PKGINFO" <<EOF
pkgname = elytra-horizon
pkgbase = elytra-horizon
pkgver = ${version}-1
pkgdesc = Launcher for Elytra Horizon (Minecraft: Java Edition rewritten in Rust)
url = https://github.com/elytra-horizon/horizon
builddate = ${builddate}
packager = Elytra Horizon <https://github.com/elytra-horizon/horizon>
size = ${size}
arch = x86_64
license = custom
depend = alsa-lib
depend = libx11
depend = libxkbcommon
depend = glibc
provides = horizon-launcher
conflicts = horizon-launcher
EOF

# .MTREE for pacman file verification if bsdtar is available
if command -v bsdtar >/dev/null 2>&1; then
  (cd "$pkgroot" && bsdtar -czf .MTREE --format=mtree \
    --options='!all,use-set,type,uid,gid,mode,time,size,md5,sha256,link' \
    .PKGINFO usr) || true
fi

pkg_name="elytra-horizon-${version}-1-x86_64.pkg.tar.zst"
if command -v zstd >/dev/null 2>&1; then
  files=(".PKGINFO")
  [[ -f "$pkgroot/.MTREE" ]] && files+=(".MTREE")
  files+=("usr")
  (cd "$pkgroot" && tar --numeric-owner --owner=0 --group=0 -cf - "${files[@]}") | zstd -T0 -19 - > "$out/$pkg_name"
  echo "Packaged $out/$pkg_name"
else
  echo "zstd not found, skipping .pkg.tar.zst generation" >&2
fi

# AUR PKGBUILD template
cat > "$out/PKGBUILD" <<EOF
# Maintainer: Elytra Horizon <https://github.com/elytra-horizon/horizon>
pkgname=elytra-horizon-bin
pkgver=${version}
pkgrel=1
pkgdesc="Launcher for Elytra Horizon (Minecraft: Java Edition rewritten in Rust)"
arch=('x86_64')
url="https://github.com/elytra-horizon/horizon"
license=('custom')
depends=('alsa-lib' 'libx11' 'libxkbcommon' 'vulkan-driver')
provides=('elytra-horizon' 'horizon-launcher')
conflicts=('elytra-horizon' 'horizon-launcher')
source=("https://github.com/elytra-horizon/horizon/releases/download/launcher@\${pkgver}/horizon-launcher-linux-x86_64.tar.gz")
sha256sums=('SKIP')

package() {
    install -Dm755 horizon-launcher "\$pkgdir/usr/bin/elytra-horizon"
    ln -s elytra-horizon "\$pkgdir/usr/bin/horizon-launcher"
}
EOF
echo "Generated $out/PKGBUILD"
