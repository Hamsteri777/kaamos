# slint-kit

Shared Slint UI kit and Linux desktop helpers (from NARME and PulsarFe). Talk to the user in Finnish.

## Layout

- `crates/ui` (`slint-kit-ui`): `.slint` files, fonts, icons; no dependencies (build-dependency).
- `crates/kit` (`slint-kit`): Rust helpers: `platform`, `tray` (feature "tray"), `autostart`,
  `paths`, `poll`.
- `crates/gallery`: every component in one window; also the usage example.
- `template/` + `new-project.sh`: project template (excluded from the workspace).

## Commands

```bash
./check.sh                      # build, test, clippy -D warnings, fmt (whole workspace)
cargo run -p slint-kit-gallery
SLINT_KIT_PATH=$PWD ./new-project.sh demo "Demo" /tmp/demo   # test the template with path deps
```

Run `./check.sh` before every commit. When the template or a public API changes, also generate a
project from the template and run its `check.sh`.

## Rules

- Keep the theme and components in sync with what NARME/PulsarFe need; do not change colours or
  sizes without the user's approval (both apps should keep their look).
- Releases are git tags (`v0.1.0`, …); apps depend on a tag. Bump the tag in `template/Cargo.toml`
  and README when releasing.
- Slint is pinned to 1.18 because `platform` uses the unstable winit 0.30 API.
