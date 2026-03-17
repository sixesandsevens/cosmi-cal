# CosmiCal

A minimal calendar, notes, and clipboard utility for the COSMIC desktop

## Local install

Install CosmiCal into your user account without root:

```sh
./scripts/install-local.sh
```

This installs:

- `~/.local/bin/cosmi-cal`
- `~/.local/share/applications/io.github.sixesandsevens.cosmical.desktop`
- `~/.local/share/icons/hicolor/256x256/apps/io.github.sixesandsevens.cosmical.png`

After installation, launch `CosmiCal` from your app launcher or run:

```sh
cosmi-cal
```

To remove the user-local install:

```sh
./scripts/uninstall-local.sh
```

## Summon shortcut

CosmiCal supports a first-pass summon mode intended for COSMIC keyboard shortcuts:

```sh
cosmi-cal --summon
```

Bind Super + C in COSMIC Settings to:

```sh
cosmi-cal --summon
```

Behavior:

- if CosmiCal is not running, it launches in surface mode
- if it is already running, the existing window is focused and the today note is targeted

## Development

A [justfile](./justfile) is included for the [casey/just][just] command runner.

- `just` builds the application with the default `just build-release` recipe
- `just run` builds and runs the application
- `just install` installs the project into the system
- `just vendor` creates a vendored tarball
- `just build-vendored` compiles with vendored dependencies from that tarball
- `just check` runs clippy on the project to check for linter warnings
- `just check-json` can be used by IDEs that support LSP

## Translators

[Fluent][fluent] is used for localization of the software. Fluent's translation files are found in the [i18n directory](./i18n). New translations may copy the [English (en) localization](./i18n/en) of the project, rename `en` to the desired [ISO 639-1 language code][iso-codes], and then translations can be provided for each [message identifier][fluent-guide]. If no translation is necessary, the message may be omitted.

## Packaging

If packaging for a Linux distribution, vendor dependencies locally with the `vendor` rule, and build with the vendored sources using the `build-vendored` rule. When installing files, use the `rootdir` and `prefix` variables to change installation paths.

```sh
just vendor
just build-vendored
just rootdir=debian/cosmi-cal prefix=/usr install
```

It is recommended to build a source tarball with the vendored dependencies, which can typically be done by running `just vendor` on the host system before it enters the build environment.

## Developers

Developers should install [rustup][rustup] and configure their editor to use [rust-analyzer][rust-analyzer]. To improve compilation times, disable LTO in the release profile, install the [mold][mold] linker, and configure [sccache][sccache] for use with Rust. The [mold][mold] linker will only improve link times if LTO is disabled.

[fluent]: https://projectfluent.org/
[fluent-guide]: https://projectfluent.org/fluent/guide/hello.html
[iso-codes]: https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes
[just]: https://github.com/casey/just
[rustup]: https://rustup.rs/
[rust-analyzer]: https://rust-analyzer.github.io/
[mold]: https://github.com/rui314/mold
[sccache]: https://github.com/mozilla/sccache
