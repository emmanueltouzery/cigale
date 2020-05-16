cd "$(git rev-parse --show-toplevel)" || exit 1
cp flatpak/com.github.emmanueltouzery.cigale.json .
cp flatpak/com.github.emmanueltouzery.cigale.metainfo.xml .
mkdir .cargo
cargo vendor > .cargo/config
rm flatpak-cargo-generator.py
wget https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/2982bbca0bef22559325dacdd7847fb5e3e0e332/cargo/flatpak-cargo-generator.py
python3 flatpak-cargo-generator.py ./Cargo.lock -o cargo-sources.json
flatpak-builder --install repo com.github.emmanueltouzery.cigale.json --force-clean --user
rm flatpak-cargo-generator.py
rm com.github.emmanueltouzery.cigale.json
rm com.github.emmanueltouzery.cigale.metainfo.xml
rm cargo-sources.json
rm -Rf vendor
