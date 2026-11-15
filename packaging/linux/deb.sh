#!/usr/bin/env bash
# Packages the Linux launcher as a Debian/Ubuntu .deb package for Elytra Horizon.
#
#   packaging/linux/deb.sh BINARY VERSION OUTDIR
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
         "$pkgroot/usr/share/icons" \
         "$pkgroot/DEBIAN"

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

# Icons: install hicolor icons for both elytra-horizon and horizon
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

# Calculate installed size in KiB
size=$(du -ks "$pkgroot" | cut -f1)

cat > "$pkgroot/DEBIAN/control" <<EOF
Package: elytra-horizon
Version: ${version}
Section: games
Priority: optional
Architecture: amd64
Maintainer: Elytra Horizon <https://github.com/elytra-horizon/horizon>
Depends: libc6 (>= 2.31), libasound2, libx11-6, libxkbcommon0
Provides: horizon-launcher
Conflicts: horizon-launcher
Replaces: horizon-launcher
Installed-Size: ${size}
Description: Launcher for Elytra Horizon (Minecraft: Java Edition rewritten in Rust)
 Elytra Horizon Launcher installs, updates, and launches Horizon,
 Minecraft: Java Edition 26.3 rewritten in Rust with wgpu.
EOF

deb_name="elytra-horizon_${version}_amd64.deb"
dpkg-deb --build --root-owner-group "$pkgroot" "$out/$deb_name"

echo "Packaged $out/$deb_name"
