# Maintainer: Your Name <you@example.com>
pkgname=arch-suite
pkgver=1.5.0
pkgrel=1
pkgdesc="A TUI-driven suite to replicate, clone, and manage Arch Linux systems."
arch=('x86_64')
url="https://github.com/sarang-kernel/arch-suite.git"
license=('MIT')
# CORRECTED: 'lspci' is replaced with its package 'pciutils'.
# All dependencies are now valid package names.
depends=(
    'bash'
    'gum'
    'arch-install-scripts'
    'pacman-contrib'
    'gptfdisk'
    'dosfstools'
    'e2fsprogs'
    'archiso'
    'rsync'
    'pciutils' 
)
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver::git+file://$PWD")
sha256sums=('SKIP')

build() {
    cd "$srcdir/$pkgname-$pkgver/tui-launcher"
    
    # This is the crucial step: Define the final path and export it for Rust to use.
    export ENGINE_SCRIPT_PATH="/usr/share/$pkgname/engine.sh"
    
    # Build the release binary
    cargo build --release --locked
}

package() {
    # Install the compiled Rust binary, renaming it to the package name
    install -Dm755 "$srcdir/$pkgname-$pkgver/tui-launcher/target/release/tui-launcher" "$pkgdir/usr/bin/$pkgname"
    
    # Install the Bash engine script
    install -Dm755 "$srcdir/$pkgname-$pkgver/engine.sh" "$pkgdir/usr/share/$pkgname/engine.sh"
    
    # Install the license file
    install -Dm644 "$srcdir/$pkgname-$pkgver/LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
