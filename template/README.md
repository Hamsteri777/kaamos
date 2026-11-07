# __TITLE__

Built with Rust, Slint and [Kaamos](https://github.com/Hamsteri777/kaamos).

```bash
./check.sh          # build, test, clippy, fmt
cargo run           # window + tray
cargo run -- --tray # start in the tray
```

Arch package: `cd packaging/arch && makepkg -si`. It builds the committed HEAD with
`cargo fetch --locked`, so commit `Cargo.lock` (created by the first build).

The app icon `packaging/__APP__.svg` is a placeholder: replace it with your own.

Licence: GPL-3.0-or-later. Third-party notices (Inter font, Feather icons): `THIRD_PARTY_NOTICES`
and `licenses/`.
