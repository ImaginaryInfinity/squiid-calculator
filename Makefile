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

build: require
	cargo build --release

appimage: require clean build
	# Check for appimagetool
	@appimagetool --version > /dev/null 2>&1 || (echo "ERROR: appimagetool is required"; exit 1)

	# Make directory structure
	mkdir -p package-build/squiid.AppDir/usr/bin
	# Copy squiid binary
	cp target/release/squiid package-build/squiid.AppDir/usr/bin/squiid
	# Copy AppRun
	cp packages/appimage/AppRun package-build/squiid.AppDir/AppRun
	# Make AppRun executable
	chmod +x package-build/squiid.AppDir/AppRun
	# Copy desktop file
	cp packages/appimage/squiid.desktop package-build/squiid.AppDir/squiid.desktop
	# Copy icon
	cp branding/squiidsquareblack.png package-build/squiid.AppDir/icon.png
	# Build appimage
	appimagetool package-build/squiid.AppDir package-build/Squiid_Calculator.AppImage
