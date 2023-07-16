.DEFAULT_GOAL:=help

PREFIX ?= /usr
BINDIR ?= $(PREFIX)/bin
BINARY_NAME := squiid
BINARY_PATH ?= target/release/$(BINARY_NAME)

DEBUILD_OPTIONS ?= -us -uc

APPIMAGETOOL ?= appimagetool

VERSION := $(shell awk 'sub(/^[[:space:]]*version[[:space:]]*=[[:space:]]*/, "") {sub(/^"/, ""); sub(/".*$$/, ""); print}' Cargo.toml)
export VERSION

.PHONY: help
help: ## Shows this help message
	@awk 'BEGIN { \
		# Set the field separator (FS) to ":.*##" \
		FS = ":.*##"; \
		# Print a usage message with a highlighted <target> placeholder \
		printf "Usage: make \033[36m<target>\033[0m\n" \
	} \
	/^[a-zA-Z_-]+:.*?##/ { \
		# If the line matches the pattern for a target and its description, \
		# print the target name and description in a formatted string \
		printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2 \
	} \
	/^##@/ { \
		# If the line matches the pattern for a section header, \
		# print it in bold font \
		printf "\n\033[1m%s\033[0m\n", substr($$0, 5) \
	}' $(MAKEFILE_LIST)

clean: ## Clean the build environment
	rm -rf package-build \
		org.imaginaryinfinity.Squiid.json \
		generated-sources.json \
		.flatpak-builder \
		flatpak-cargo-generator.py \
		../squiid_0.1.0.orig.tar.gz \
		debian

require:
	@echo "Checking the programs required for the build are installed..."
	@cargo --version >/dev/null 2>&1 || (echo "ERROR: cargo is required."; exit 1)

test: ## Test each component of the project
	cargo test -p squiid-parser -p squiid-engine -p squiid

build: require ## Build the release version of the program for the system platform
	cargo build --release

build-musl: require ## Build the Linux MUSL version
	cargo build --release --target=x86_64-unknown-linux-musl

install: $(BINARY_PATH) ## Install Squiid to the system
	mkdir -p $(DESTDIR)$(BINDIR)
	install -v -m755 $(BINARY_PATH) $(DESTDIR)$(BINDIR)

uninstall: ## Uninstall the version of Squiid installed with the Makefile
	rm $(DESTDIR)$(BINDIR)/$(BINARY_NAME)

flatpak: require clean ## Build the flatpak in package-build/
	@python3 --version >/dev/null 2>&1 || (echo "ERROR: python3 is required."; exit 1)
	@flatpak-builder --version >/dev/null 2>&1 || (echo "ERROR: flatpak-builder is required."; exit 1)
	@curl --version >/dev/null 2>&1 || (echo "ERROR: curl is required."; exit 1)

	mkdir -p package-build

	curl https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py -Lo flatpak-cargo-generator.py

	python3 flatpak-cargo-generator.py ./Cargo.lock -o generated-sources.json

	cp packages/flatpak/org.imaginaryinfinity.Squiid.json ./

	flatpak-builder package-build org.imaginaryinfinity.Squiid.json

	rm -f org.imaginaryinfinity.Squiid.json generated-sources.json flatpak-cargo-generator.py

snap: require clean ## Build the snap
	@snapcraft --version >/dev/null 2>&1 || (echo "ERROR: snapcraft is required."; exit 1)
	@envsubst --version >/dev/null 2>&1 || (echo "ERROR: envsubst is required."; exit 1)

	@echo "Replacing VERSION with ${VERSION} in snapcraft.yaml"
	@envsubst '$${VERSION}' < packages/snap/snapcraft.yaml > snapcraft.yaml

	snapcraft

	rm -f snapcraft.yaml

appimage: require clean build ## Build the AppImage
	# Check for appimagetool
	@$(APPIMAGETOOL) --version > /dev/null 2>&1 || (echo "ERROR: appimagetool is required"; exit 1)
	# Check for curl
	@curl --version > /dev/null 2>&1 || (echo "ERROR: curl is required"; exit 1)
	# check for envsubst
	@envsubst --version >/dev/null 2>&1 || (echo "ERROR: envsubst is required."; exit 1)

	# Make directory structure
	mkdir -p package-build/squiid.AppDir/usr/bin
	mkdir -p package-build/squiid.AppDir/usr/share/icons
	# Copy squiid binary
	cp target/release/squiid package-build/squiid.AppDir/usr/bin/squiid
	# Copy AppRun
	cp packages/appimage/AppRun package-build/squiid.AppDir/AppRun
	# Make AppRun executable
	chmod +x package-build/squiid.AppDir/AppRun
	# Copy and format desktop file
	@envsubst '$${VERSION}' < packages/appimage/squiid.desktop > package-build/squiid.AppDir/squiid.desktop
	# Copy icon
	cp branding/squiidsquare.png package-build/squiid.AppDir/squiid.png
	cp branding/squiidsquare.png package-build/squiid.AppDir/usr/share/icons/squiid.png
	# Download and add kitty terminal to appimage
	curl -L https://github.com/kovidgoyal/kitty/releases/download/v0.27.1/kitty-0.27.1-x86_64.txz -o package-build/kitty.txz
	# Untar kitty
	tar -xf package-build/kitty.txz --directory package-build/squiid.AppDir/usr/
	# Make sure kitty is executable
	chmod +x package-build/squiid.AppDir/usr/bin/kitty
	# Copy kitty config
	cp packages/appimage/kitty.conf package-build/squiid.AppDir/kitty.conf
	# Remove unneeded kitty components
	rm package-build/squiid.AppDir/usr/bin/kitten
	rm -rf package-build/squiid.AppDir/usr/share/applications
	rm -rf package-build/squiid.AppDir/usr/share/doc
	rm -rf package-build/squiid.AppDir/usr/share/man
	rm -rf package-build/squiid.AppDir/usr/share/icons
	# Build appimage
	$(APPIMAGETOOL) package-build/squiid.AppDir package-build/Squiid_Calculator.AppImage

windows-build: require clean ## Cross compile the Windows release
	# cross compile windows version
	cargo build --release --target=x86_64-pc-windows-gnu

windows-installer: windows-build ## Build the Windows installer
	# Check for docker
	@docker --version > /dev/null 2>&1 || (echo "ERROR: docker is required"; exit 1)
	@envsubst --version >/dev/null 2>&1 || (echo "ERROR: envsubst is required."; exit 1)

	# bundle assets
	mkdir package-build
	cp packages/windows/squiid.iss package-build/
	@envsubst '$${VERSION}' < packages/windows/squiid.iss > package-build/squiid.iss
	cp branding/squiidsquare.ico package-build/
	cp LICENSE package-build/LICENSE.txt
	cp target/x86_64-pc-windows-gnu/release/squiid.exe package-build

	# build the windows installer with an output directory of the current directory
	docker run --rm -i -v "$$PWD/package-build:/work" amake/innosetup squiid.iss /O.\\

# ANDROID
# TODO: fix android building
android-require: require
ifndef platform
	# check if platform= argument is defined
	@echo "ERROR: platform is not defined. please specify an android ndk version with `platform=xx` (for example, 33)"
	exit 1
endif
	# check if cargo ndk is installed
	@cargo ndk --version > /dev/null 2>&1 || (echo "ERROR: cargo-ndk is required. Install it with `cargo install cargo-ndk`"; exit 1)

android-armv8: export TARGET_CMAKE_TOOLCHAIN_FILE=/opt/android-ndk/build/cmake/android.toolchain.cmake
android-armv8: android-require ## Build the Android ARMv8 release
	RUST_LOG=debug cargo ndk --platform $(platform) --target arm64-v8a build --release

android-armv7: export TARGET_CMAKE_TOOLCHAIN_FILE=/opt/android-ndk/build/cmake/android.toolchain.cmake
android-armv7: android-require ## Build the Android ARMv7 release
	cargo ndk --platform $(platform) --target armeabi-v7a build --release

android-x86_64: export TARGET_CMAKE_TOOLCHAIN_FILE=/opt/android-ndk/build/cmake/android.toolchain.cmake
android-x86_64: android-require ## Build the Android x86_64 release
	cargo ndk --platform $(platform) --target x86_64 build --release

android: export TARGET_CMAKE_TOOLCHAIN_FILE=/opt/android-ndk/build/cmake/android.toolchain.cmake
android: android-armv8 android-armv7 android-x86_64 ## Build all android targets

aur-metadata: require clean ## Build the AUR metadata files for deployment
	# check for makepkg
	@makepkg --version > /dev/null 2>&1 || (echo "ERROR: makepkg is required"; exit 1)
	@envsubst --version >/dev/null 2>&1 || (echo "ERROR: envsubst is required."; exit 1)

	mkdir -p package-build/
	@envsubst '$${VERSION}' < packages/arch/PKGBUILD > package-build/PKGBUILD
	# retrieve sha512sum of source
	export SHA512SUM=$$(curl -sL $$(cd package-build; makepkg --printsrcinfo | makepkg --printsrcinfo | grep -oP 'source = \K.*') | sha512sum | awk '{print $$1}'); \
	envsubst '$${SHA512SUM}' < package-build/PKGBUILD > package-build/PKGBUILD-new

	mv package-build/PKGBUILD-new package-build/PKGBUILD

	cd package-build; makepkg --printsrcinfo > .SRCINFO

arch-package: aur-metadata ## Build an Arch package
	cd package-build; makepkg -s

deb: require clean
	@git --version > /dev/null 2>&1 || (echo "ERROR: git is required"; exit 1)
	@debuild --version > /dev/null 2>&1 || (echo "ERROR: debuild is required"; exit 1)

	ls packages

	mkdir -p package-build
	cp -r packages/debian ./

	git archive --format=tar.gz -o ../squiid_0.1.0.orig.tar.gz trunk

	debuild $(DEBUILD_OPTIONS)

	ls ..

	mv ../squiid*.deb ../squiid*.build ../squiid*.changes ../squiid*.tar.xz ../squiid*.dsc ../squiid*.buildinfo ./package-build || true

	rm -rf ../squiid_0.1.0.orig.tar.gz debian

rpm: require clean
	@envsubst --version >/dev/null 2>&1 || (echo "ERROR: envsubst is required."; exit 1)

	mkdir -p package-build

	@envsubst '$${VERSION}' < packages/fedora/squiid.spec > package-build/squiid.spec