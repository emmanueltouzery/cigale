cd "$(git rev-parse --show-toplevel)" || exit 1
cp flatpak/com.github.emmanueltouzery.cigale.json .
cp flatpak/com.github.emmanueltouzery.cigale.metainfo.xml .
mkdir .cargo
cargo vendor > .cargo/config
rm flatpak-cargo-generator.py
wget https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/d7cfbeaf8d1a2165d917d048511353d6f6e59ab3/cargo/flatpak-cargo-generator.py
python3 flatpak-cargo-generator.py ./Cargo.lock -o cargo-sources.json
flatpak-builder --install repo com.github.emmanueltouzery.cigale.json --force-clean --user
rm flatpak-cargo-generator.py
rm com.github.emmanueltouzery.cigale.json
rm com.github.emmanueltouzery.cigale.metainfo.xml
rm cargo-sources.json
