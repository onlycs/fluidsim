.PHONY: dist dist-linux dist-win dist-osx

dist: dist-linux dist-win dist-osx

dist-linux:
	mkdir -p dist
	cargo build --target x86_64-unknown-linux-gnu -p fluidsim --release
	cp target/x86_64-unknown-linux-gnu/release/fluidsim dist/fluidsim-linux-x86_64
	patchelf --remove-rpath dist/fluidsim-linux-x86_64
	xz -9 dist/fluidsim-linux-x86_64

dist-win:
	mkdir -p dist
	cargo xwin build --target x86_64-pc-windows-msvc -p fluidsim --release
	cp target/x86_64-pc-windows-msvc/release/fluidsim.exe dist/fluidsim-windows-x86_64.exe

dist-osx:
	mkdir -p dist
	SDKROOT=$(SDKROOT) cargo zigbuild --target universal2-apple-darwin -p fluidsim --release
	cp target/universal2-apple-darwin/release/fluidsim osx/FluidSim.app/Contents/MacOS/FluidSim
	zip -r dist/fluidsim-osx-universal.zip osx/FluidSim.app
