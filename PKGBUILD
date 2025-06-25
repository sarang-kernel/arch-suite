# Maintainer: Sarang Vehale sarang-kernel@gmail.com
pkgname=arch-suite
pkgver=2.1.0
pkgrel=1
pkgdesc="An intuitive, TUI-driven suite to replicate, clone, and manage Arch Linux systems."
arch=('x86_64')
url="https://github.com/sarang-kernel/arch-suite.git"
license=('MIT')
# These are now the RUNTIME dependencies for the binary.
depends=(
    'bash' # Still needed to run commands in chroot
    'gum'  # Still the easiest way to get user input inside command logic
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
    cd "$srcdir/$pkgname-$pkgver"
    # No environment variables needed anymore.
    cargo build --release --locked
}

package() {
    # Install the single compiled Rust binary.
    install -Dm755 "$srcdir/$pkgname-$pkgver/target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    
    # Install the license file.
    install -Dm644 "$srcdir/$pkgname-$pkgver/LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
