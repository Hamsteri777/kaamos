# __TITLE__

Talk to the user in Finnish.

## Stack

- Rust (stable, edition 2024), GUI in Slint 1.18 with **slint-kit**
  (github.com/Hamsteri777/slint-kit): `@kit/kit.slint` for the theme and components,
  `slint_kit` for the Linux helpers.
- `src/main.rs`: worker thread + UI wiring. `ui/main.slint`: the window.

## Commands

```bash
./check.sh   # cargo build, cargo test, cargo clippy -D warnings, cargo fmt --check
```

Run it before reporting a task as done and before every commit.

## slint-kit rules (learned the hard way, keep them)

- Call `slint_kit::platform::init(APP_ID)` first in `main`; `APP_ID` = the `.desktop` file name.
  Otherwise the task bar shows the window and the launcher separately and leaves an empty
  "starting" entry.
- Tray: `slint_kit::tray` (ksni). Do not use Slint's `SystemTrayIcon`: it creates an extra empty
  window on Wayland. With a tray, use `run_event_loop_until_quit` and quit from the tray menu.
- (i) infos: `SettingRow { info-body: "…"; }` / `Section { info-body: "…"; }`, and keep
  `InfoLayer { }` as the **last** child of the Window.
- The UI never waits for slow work: send commands to the worker thread, receive replies with
  `slint_kit::poll::drain_every`.
- Std widgets use the `fluent-dark` style (build.rs). Use `Theme.*` colours, not literals.

## Packaging

`packaging/arch/PKGBUILD` builds the committed HEAD: the user runs `cd packaging/arch && makepkg -si`.
Claude does not run `makepkg -si` or sudo.
