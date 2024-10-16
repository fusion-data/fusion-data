# Backend

## Development Setup

MacOS:

```bash
brew install llvm
```

Linux:

- Rocky/Fedora, `sudo dnf install lld clang`
- Debian/Ubuntu, `sudo apt-get install lld clang`
- Arch, `sudo pacman -S lld clang`

Windows:

```powershell
cargo install -f cargo-binutils
rustup component add llvm-tools-preview
```
