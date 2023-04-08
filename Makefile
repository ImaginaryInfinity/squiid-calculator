PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
BINARY_NAME := squiid
BINARY_PATH := target/release/$(BINARY_NAME)

VERSION := $(shell tomlq -r '.package.version' Cargo.toml)
export VERSION

clean:
	rm -rf package-build \
		org.imaginaryinfinity.Squiid.json \
		generated-sources.json \
		.flatpak-builder

require:
	@echo "Checking the programs required for the build are installed..."
	@cargo --version >/dev/null 2>&1 || (echo "ERROR: cargo is required."; exit 1)

test:
	cargo test -p squiid-parser -p squiid-engine -p squiid

build: require
	cargo build --release

install: $(BINARY_PATH) build
	mkdir -p $(DESTDIR)$(BINDIR)
	cp $(BINARY_PATH) $(DESTDIR)$(BINDIR)

uninstall:
	rm $(DESTDIR)$(BINDIR)/$(BINARY_NAME)

flatpak: require clean
	@python3 --version >/dev/null 2>&1 || (echo "ERROR: python3 is required."; exit 1)
	@flatpak-builder --version >/dev/null 2>&1 || (echo "ERROR: flatpak-builder is required."; exit 1)

	mkdir -p package-build

	python3 ./packages/flatpak/flatpak-cargo-generator.py ./Cargo.lock -o generated-sources.json

	cp packages/flatpak/org.imaginaryinfinity.Squiid.json ./

	flatpak-builder package-build org.imaginaryinfinity.Squiid.json

	rm -f org.imaginaryinfinity.Squiid.json generated-sources.json

snap: require clean
	@snapcraft --version >/dev/null 2>&1 || (echo "ERROR: snapcraft is required."; exit 1)
	@envsubst --version >/dev/null 2>&1 || (echo "ERROR: envsubst is required."; exit 1)

	@echo "Replacing VERSION with ${VERSION} in snapcraft.yaml"
	@envsubst '$${VERSION}' < packages/snap/snapcraft.yaml > snapcraft.yaml

	snapcraft

	rm -f snapcraft.yaml

appimage: require clean build
	# Check for appimagetool
	@appimagetool --version > /dev/null 2>&1 || (echo "ERROR: appimagetool is required"; exit 1)
	# Check for curl
	@curl --version > /dev/null 2>&1 || (echo "ERROR: curl is required"; exit 1)

	# Make directory structure
	mkdir -p package-build/squiid.AppDir/usr/bin
	mkdir -p package-build/squiid.AppDir/usr/share/icons
	# Copy squiid binary
	cp target/release/squiid package-build/squiid.AppDir/usr/bin/squiid
	# Copy AppRun
	cp packages/appimage/AppRun package-build/squiid.AppDir/AppRun
	# Make AppRun executable
	chmod +x package-build/squiid.AppDir/AppRun
	# Copy desktop file
	cp packages/appimage/squiid.desktop package-build/squiid.AppDir/squiid.desktop
	# Copy icon
	cp branding/squiidsquareblack.png package-build/squiid.AppDir/squiid.png
	cp branding/squiidsquareblack.png package-build/squiid.AppDir/usr/share/icons/squiid.png
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
	appimagetool package-build/squiid.AppDir package-build/Squiid_Calculator.AppImage
