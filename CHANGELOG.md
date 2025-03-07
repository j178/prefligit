# Changelog

## 0.0.10

### Breaking changes

**Warning**: This release changed the store layout, it's recommended to delete the old store and install from scratch.

To delete the old store, run:

```sh
rm -rf ~/.cache/prefligit
```

### Enhancements

- Restructure store folders layout ([#181](https://github.com/j178/prefligit/pull/181))
- Fallback some env vars to to pre-commit ([#175](https://github.com/j178/prefligit/pull/175))
- Save patches to `$PREFLIGIT_HOME/patches` ([#182](https://github.com/j178/prefligit/pull/182))

### Bug fixes

- Fix removing git env vars ([#176](https://github.com/j178/prefligit/pull/176))
- Fix typo in Cargo.toml ([#160](https://github.com/j178/prefligit/pull/160))

### Other changes

- Do not publish to crates.io ([#191](https://github.com/j178/prefligit/pull/191))
- Bump cargo-dist to v0.28.0 ([#170](https://github.com/j178/prefligit/pull/170))
- Bump uv version to 0.6.0 ([#184](https://github.com/j178/prefligit/pull/184))
- Configure Renovate ([#168](https://github.com/j178/prefligit/pull/168))
- Format sample config output ([#172](https://github.com/j178/prefligit/pull/172))
- Make env vars a shareable crate ([#171](https://github.com/j178/prefligit/pull/171))
- Reduce String alloc ([#166](https://github.com/j178/prefligit/pull/166))
- Skip common git flags in command trace log ([#162](https://github.com/j178/prefligit/pull/162))
- Update Rust crate clap to v4.5.29 ([#173](https://github.com/j178/prefligit/pull/173))
- Update Rust crate which to v7.0.2 ([#163](https://github.com/j178/prefligit/pull/163))
- Update astral-sh/setup-uv action to v5 ([#164](https://github.com/j178/prefligit/pull/164))
- Upgrade Rust to 1.84 and upgrade dependencies ([#161](https://github.com/j178/prefligit/pull/161))

## 0.0.9

Due to a mistake in the release process, this release is skipped.

## 0.0.8

### Enhancements

- Move home dir to `~/.cache/prefligit` ([#154](https://github.com/j178/prefligit/pull/154))
- Implement trailing-whitespace in Rust ([#137](https://github.com/j178/prefligit/pull/137))
- Limit hook install concurrency ([#145](https://github.com/j178/prefligit/pull/145))
- Simplify language default version implementation ([#150](https://github.com/j178/prefligit/pull/150))
- Support install uv from pypi ([#149](https://github.com/j178/prefligit/pull/149))
- Add executing command to error message ([#141](https://github.com/j178/prefligit/pull/141))

### Bug fixes

- Use hook `args` in fast path ([#139](https://github.com/j178/prefligit/pull/139))

### Other changes

- Remove hook install_key ([#153](https://github.com/j178/prefligit/pull/153))
- Remove pyvenv.cfg patch ([#156](https://github.com/j178/prefligit/pull/156))
- Try to use D drive on Windows CI ([#157](https://github.com/j178/prefligit/pull/157))
- Tweak trailing-whitespace-fixer ([#140](https://github.com/j178/prefligit/pull/140))
- Upgrade dist to v0.27.0 ([#158](https://github.com/j178/prefligit/pull/158))
- Uv install python into tools path ([#151](https://github.com/j178/prefligit/pull/151))

## 0.0.7

### Enhancements

- Add progress bar for hook init and install ([#122](https://github.com/j178/prefligit/pull/122))
- Add color to command help ([#131](https://github.com/j178/prefligit/pull/131))
- Add commit info to version display ([#130](https://github.com/j178/prefligit/pull/130))
- Support meta hooks reading ([#134](https://github.com/j178/prefligit/pull/134))
- Implement meta hooks ([#135](https://github.com/j178/prefligit/pull/135))

### Bug fixes

- Fix same repo clone multiple times ([#125](https://github.com/j178/prefligit/pull/125))
- Fix logging level after renaming ([#119](https://github.com/j178/prefligit/pull/119))
- Fix version tag distance ([#132](https://github.com/j178/prefligit/pull/132))

### Other changes

- Disable uv cache on Windows ([#127](https://github.com/j178/prefligit/pull/127))
- Impl Eq and Hash for ConfigRemoteRepo ([#126](https://github.com/j178/prefligit/pull/126))
- Make `pass_env_vars` runs on Windows ([#133](https://github.com/j178/prefligit/pull/133))
- Run cargo update ([#129](https://github.com/j178/prefligit/pull/129))
- Update Readme ([#128](https://github.com/j178/prefligit/pull/128))

## 0.0.6

### Breaking changes

In this release, we’ve renamed the project to `prefligit` (a deliberate misspelling of preflight) to prevent confusion with the existing pre-commit tool. For further information, refer to issue #73.

- The command-line name is now `prefligit`. We suggest uninstalling any previous version of `pre-commit-rs` and installing `prefligit` from scratch.
- The PyPI package is now listed as [`prefligit`](https://pypi.org/project/prefligit/).
- The Cargo package is also now [`prefligit`](https://crates.io/crates/prefligit).
- The Homebrew formula has been updated to `prefligit`.

### Enhancements

- Support `docker_image` language ([#113](https://github.com/j178/pre-commit-rs/pull/113))
- Support `init-templatedir` subcommand ([#101](https://github.com/j178/pre-commit-rs/pull/101))
- Implement get filenames from merge conflicts ([#103](https://github.com/j178/pre-commit-rs/pull/103))

### Bug fixes

- Fix `prefligit install --hook-type` name ([#102](https://github.com/j178/pre-commit-rs/pull/102))

### Other changes

- Apply color option to log ([#100](https://github.com/j178/pre-commit-rs/pull/100))
- Improve tests ([#106](https://github.com/j178/pre-commit-rs/pull/106))
- Remove intermedia Language enum ([#107](https://github.com/j178/pre-commit-rs/pull/107))
- Run `cargo clippy` in the dev drive workspace ([#115](https://github.com/j178/pre-commit-rs/pull/115))

## 0.0.5

### Enhancements

v0.0.4 release process was broken, so this release is a actually a re-release of v0.0.4.

- Improve subprocess trace and error output ([#92](https://github.com/j178/pre-commit-rs/pull/92))
- Stash working tree before running hooks ([#96](https://github.com/j178/pre-commit-rs/pull/96))
- Add color to command trace ([#94](https://github.com/j178/pre-commit-rs/pull/94))
- Improve hook output display ([#79](https://github.com/j178/pre-commit-rs/pull/79))
- Improve uv installation ([#78](https://github.com/j178/pre-commit-rs/pull/78))
- Support docker language ([#67](https://github.com/j178/pre-commit-rs/pull/67))

## 0.0.4

### Enhancements

- Improve subprocess trace and error output ([#92](https://github.com/j178/pre-commit-rs/pull/92))
- Stash working tree before running hooks ([#96](https://github.com/j178/pre-commit-rs/pull/96))
- Add color to command trace ([#94](https://github.com/j178/pre-commit-rs/pull/94))
- Improve hook output display ([#79](https://github.com/j178/pre-commit-rs/pull/79))
- Improve uv installation ([#78](https://github.com/j178/pre-commit-rs/pull/78))
- Support docker language ([#67](https://github.com/j178/pre-commit-rs/pull/67))

## 0.0.3

### Bug fixes

- Check uv installed after acquired lock ([#72](https://github.com/j178/pre-commit-rs/pull/72))

### Other changes

- Add copyright of the original pre-commit to LICENSE ([#74](https://github.com/j178/pre-commit-rs/pull/74))
- Add profiler ([#71](https://github.com/j178/pre-commit-rs/pull/71))
- Publish to PyPI ([#70](https://github.com/j178/pre-commit-rs/pull/70))
- Publish to crates.io ([#75](https://github.com/j178/pre-commit-rs/pull/75))
- Rename pypi package to `pre-commit-rusty` ([#76](https://github.com/j178/pre-commit-rs/pull/76))

## 0.0.2

### Enhancements

- Add `pre-commit self update` ([#68](https://github.com/j178/pre-commit-rs/pull/68))
- Auto install uv ([#66](https://github.com/j178/pre-commit-rs/pull/66))
- Generate shell completion ([#20](https://github.com/j178/pre-commit-rs/pull/20))
- Implement `pre-commit clean` ([#24](https://github.com/j178/pre-commit-rs/pull/24))
- Implement `pre-commit install` ([#28](https://github.com/j178/pre-commit-rs/pull/28))
- Implement `pre-commit sample-config` ([#37](https://github.com/j178/pre-commit-rs/pull/37))
- Implement `pre-commit uninstall` ([#36](https://github.com/j178/pre-commit-rs/pull/36))
- Implement `pre-commit validate-config` ([#25](https://github.com/j178/pre-commit-rs/pull/25))
- Implement `pre-commit validate-manifest` ([#26](https://github.com/j178/pre-commit-rs/pull/26))
- Implement basic `pre-commit hook-impl` ([#63](https://github.com/j178/pre-commit-rs/pull/63))
- Partition filenames and delegate to multiple subprocesses ([#7](https://github.com/j178/pre-commit-rs/pull/7))
- Refactor xargs ([#8](https://github.com/j178/pre-commit-rs/pull/8))
- Skip empty config argument ([#64](https://github.com/j178/pre-commit-rs/pull/64))
- Use `fancy-regex` ([#62](https://github.com/j178/pre-commit-rs/pull/62))
- feat: add fail language support ([#60](https://github.com/j178/pre-commit-rs/pull/60))

### Bug Fixes

- Fix stage operate_on_files ([#65](https://github.com/j178/pre-commit-rs/pull/65))
