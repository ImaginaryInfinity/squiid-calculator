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
