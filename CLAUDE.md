# Kaamos

Shared Slint UI kit and Linux desktop helpers (from NARME and PulsarFe). Talk to the user in Finnish.

## Layout

- `crates/ui` (`kaamos-ui`): `.slint` files, fonts, icons; no dependencies (build-dependency).
- `crates/kaamos` (`kaamos`): Rust helpers: `platform`, `tray` (feature "tray"), `autostart`,
  `paths`, `poll`.
- `crates/gallery`: every component in one window; also the usage example.
- `template/` + `new-project.sh`: project template (excluded from the workspace).

## Commands

```bash
./check.sh                      # build, test, clippy -D warnings, fmt (whole workspace)
cargo run -p kaamos-gallery
KAAMOS_PATH=$PWD ./new-project.sh demo "Demo" /tmp/demo   # test the template with path deps
```

Run `./check.sh` before every commit. When the template or a public API changes, also generate a
project from the template and run its `check.sh`.

## Rules

- Keep the theme and components in sync with what NARME/PulsarFe need; do not change colours or
  sizes without the user's approval (both apps should keep their look).
- Releases are git tags (`v0.1.0`, …); apps depend on a tag. When releasing, bump
  `[workspace.package] version` in `Cargo.toml`, the version chip in `crates/gallery/ui/gallery.slint`
  and the tag in `template/Cargo.toml` and README, run `./check.sh` and the template test, then tag.
- Slint is held to the 1.18 series (`"~1.18.1"`, not `"1.18.1"`, which would allow 1.19) because
  `platform` uses the unstable winit 0.30 API. Apps must use `~` too.
