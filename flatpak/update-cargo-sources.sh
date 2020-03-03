# must have a checkout from the flathub repo in ../flathub of the cigale checkout folder
cd "$(git rev-parse --show-toplevel)" || exit 1
mkdir .cargo
cargo vendor > .cargo/config
rm flatpak-cargo-generator.py
wget https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/d7cfbeaf8d1a2165d917d048511353d6f6e59ab3/cargo/flatpak-cargo-generator.py
python3 flatpak-cargo-generator.py Cargo.lock -o ../flathub/cargo-sources.json
