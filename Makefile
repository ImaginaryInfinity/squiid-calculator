clean:
	rm -rf package-build
	rm -f org.imaginaryinfinity.Squiid.json generated-sources.json

require:
	@echo "Checking the programs required for the build are installed..."
	@cargo --version >/dev/null 2>&1 || (echo "ERROR: cargo is required."; exit 1)

test:
	cargo test -p squiid-parser -p squiid-engine -p squiid

flatpak: require test clean
	@python3 --version >/dev/null 2>&1 || (echo "ERROR: python3 is required."; exit 1)
	@flatpak-builder --version >/dev/null 2>&1 || (echo "ERROR: flatpak-builder is required."; exit 1)

	mkdir -p package-build

	python3 ./packages/flatpak/flatpak-cargo-generator.py ./Cargo.lock -o generated-sources.json

	cp packages/flatpak/org.imaginaryinfinity.Squiid.yml ./

	# flatpak-builder package-build org.imaginaryinfinity.Squiid.yml
	flatpak-builder --user --install --force-clean package-build org.imaginaryinfinity.Squiid.yml

	rm -f org.imaginaryinfinity.Squiid.yml generated-sources.json