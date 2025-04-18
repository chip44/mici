pkgname=mici
pkgver=0.1.0
pkgrel=1
pkgdesc="A simple microphone usage indicator for PipeWire"
arch=('any')
url="https://github.com/chip44/mici"
license=('MIT')
depends=('dbus' 'pipewire')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/$pkgver.tar.gz")
sha512sums=('SKIP')

prepare() {
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    cd "$pkgname-$pkgver"

    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --no-default-features
}

package() {
    cd "$pkgname-$pkgver"

    install -Dm755 "target/release/$pkgname" -t "$pkgdir/usr/bin/"

    for i in 16 32 48 64; do
        install -Dm644 "icons/$i.png" "$pkgdir/usr/share/icons/hicolor/${i}x${i}/apps/$pkgname.png"
    done

    # install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
    # install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
