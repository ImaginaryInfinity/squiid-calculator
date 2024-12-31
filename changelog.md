# 1.2.0

- Update some flatpak metadata issues
- Update copyright year in app.rs
- Fix a typo in the documentation
- Fix icon not showing on installed Windows version (#68)
- Move the config from the backend into the frontend (#55)
- Remove NNG in favor of FFI bindings (#70)
  - The `nng` and `ipc` features have been removed and replaced with an `ffi` feature.
- Remove `commands` command in favor of `get_stack` (Rust) and `get_stack_exposed` (FFI)
- Document `#phi` constant
- Change internal constant case from SCREAMING_SNAKE_CASE to PascalCase
- Fix some new clippy lints
- Properly gate crash reporting behind a feature flag
- Delete engine binary target as it's no longer needed without NNG
- Switch from `lazy_static` to the new `LazyLock`
- Fix musl cdylib building in the engine and the parser
- Gate parser logging behind feature flag
- Write Python, C, and C++ bindings to the engine and the parser (#29)
- Rename `MessageAction` to `EngineSignal`
  - This includes renaming the enum variants, such as `SendStack` to `StackUpdated` to reflect the removal of NNG
  - `SendCommands` and `SendPrevAnswer` have been removed as they are no longer needed due to the introduction of `get_commands` and `get_previous_answer`/`update_previous_answer`
  - A new `EngineSignal::NOP` variant was introduced for operations that don't effect the engine
- Removed the `commands` and `update_previous_answer` commands as these have been replaced with functions in the engine
- Fix a bug preventing most of the Constant PI variants from being recognized in the calculator (f4a9e9f629cd3a2a5810f0ef593e9662b2d1f0fb)
- Remove `ConstantTypes::Tau` and just have `#tau` map to `ConstantTypes::TwoPi`
- Update to Winget manifest v6 (#69)
- Update some flatpak metadata issues
- Update copyright year in app.rs
- Fix a typo in the documentation
- Fix icon not showing on installed Windows version (#68)
- Move the config from the backend into the frontend (#55)

# 1.1.3

- Fix unchecked division in `blog`
- Fix incorrect icon being installed by Makefile
- Fix overflows being unhandled

# 1.1.2

- Fix some flatpak issues with the new manifest requirements
- Update dependencies in hopes of fixing the AUR package
- Make CI (hopefully) only run when there are changes (#65)
- Make info screen fancier (98e94d57c886a73f0e68a526d190fe1d4097790b)

# 1.1.1

- Hotfix release to fix `quit` command (#57)

# 1.1.0

- Renamed `elt` and `egt` to `leq` and `geq` (#38)
- Created flatpak (#43)
- Updated documentation theme
- Fixed `make install` (#39)
- Fix previous answer not working (616edfc140a461982e195e98749d8f0d87673d30)
- Moved the config handler to the backend (#40)
- Support for typing numbers longer than the input box (#24)
- Create system for generating crash reports (#37)
- Added the `redo` command (#54)
- Added keybinds for `undo` and `redo` (#54)
- Added new tests for better coverage (#56)
- Added abstraction layer to easily drop in different IPC backends (#52)
- Update short and long descriptions (#53)
- Created deb package (#45)
- Created homebrew package (#44)

# 1.0.6

- Update Windows installer to add desktop shortcut by default for winget
- Windows installer adds the binary to the PATH variable
- Patches for flatpak

# 1.0.5

- Patch release for flatpak

# 1.0.4

- Patch release for flatpak

# 1.0.3

- Patch release for flatpak

# 1.0.2

- Patch release for flatpak

# 1.0.1

- Add command line argument parsing

# 1.0.0

Initial release. Includes a functioning TUI frontend and engine/parser components
