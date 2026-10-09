# Maintainer: Sunfalll-alt <https://github.com/Sunfalll-alt>
pkgname=solard
pkgver=0.1.0
pkgrel=1
pkgdesc="Dynamic solar and scheduled theme switcher daemon for Linux (GNOME, KDE, XFCE, Cinnamon, MATE, Kitty, Alacritty, Foot, VS Code, Neovim)"
arch=('x86_64')
url="https://github.com/Sunfalll-alt/solard"
license=('MIT')
depends=('gcc-libs' 'curl')
makedepends=('cargo')
optdepends=(
    'gammastep: Blue-light temperature control'
    'libnotify: Desktop notifications via notify-send'
    'kitty: Kitty terminal emulator theme switching'
    'alacritty: Alacritty terminal emulator theme switching'
    'foot: Foot Wayland terminal emulator theme switching'
    'code: Visual Studio Code theme switching'
    'neovim: Neovim background switching via RPC'
)
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

prepare() {
    cd "$srcdir/$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    cd "$srcdir/$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --all-targets
}

check() {
    cd "$srcdir/$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen
}

package() {
    cd "$srcdir/$pkgname-$pkgver"
    install -Dm755 "target/release/solard" "$pkgdir/usr/bin/solard"
    install -Dm644 "systemd/solard.service" "$pkgdir/usr/lib/systemd/user/solard.service"
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
