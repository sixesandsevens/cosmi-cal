# CosmiCal

CosmiCal is a small COSMIC desktop utility for keeping daily notes, a scratchpad, and quick clipboard snippets close at hand.

It is designed to be summoned like a panel: show it when you need it, dismiss it when you do not.

## Features

- Today-focused note surface (fast place to dump context)
- Scratchpad (separate from the day note)
- Clipboard helper (lightweight snippets you want to keep around)
- Summon mode for keyboard shortcuts (toggle-on / toggle-off)
- Localization via Fluent

## Install (User-Local, No Root)

Install CosmiCal into your user account:

```sh
./scripts/install-local.sh
```

This installs:

- `~/.local/bin/cosmi-cal`
- `~/.local/share/applications/io.github.sixesandsevens.cosmical.desktop`
- icons under `~/.local/share/icons/hicolor/...`

Launch `CosmiCal` from your app launcher or run:

```sh
cosmi-cal
```

Uninstall:

```sh
./scripts/uninstall-local.sh
```

## Autostart

Enable on login:

```sh
./scripts/enable-autostart.sh
```

Disable:

```sh
./scripts/disable-autostart.sh
```

## Summon Shortcut

CosmiCal supports a first-pass summon mode intended for COSMIC keyboard shortcuts:

```sh
cosmi-cal --summon
```

Suggested bindings in COSMIC Settings:

- `Super + C`: `cosmi-cal --summon`
- `Super + Shift + C`: `cosmi-cal --summon --focus-scratchpad`

Summon behavior:

- If CosmiCal is not running, it launches.
- If it is running and unfocused, the existing window is presented and focused.
- If it is running and focused, summon minimizes it (toggle-off).
- Pressing `Esc` while focused dismisses it the same way.

## Development

This is a Rust + libcosmic app.

A [justfile](./justfile) is included for the [casey/just][just] command runner.

Common tasks:

```sh
just run
just build-release
just check
```

(You can also use `cargo run` directly, but the `just` recipes are the intended workflow.)

## Translators

[Fluent][fluent] is used for localization. Translations live under [i18n](./i18n).

## Packaging

For distro packaging, the `just vendor` and `just build-vendored` recipes can build against vendored dependencies.

[fluent]: https://projectfluent.org/
[just]: https://github.com/casey/just
