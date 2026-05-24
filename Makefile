.PHONY: all linux win osx

all: linux win osx

linux:
	mkdir -p dist
	cargo build --target x86_64-unknown-linux-gnu --release
	mv target/x86_64-unknown-linux-gnu/release/fluidsim dist/fluidsim-linux-x86_64

win:
	mkdir -p dist
	cargo xwin build --target x86_64-pc-windows-msvc --release
	mv target/x86_64-pc-windows-msvc/release/fluidsim.exe dist/fluidsim-windows-x86_64.exe

osx:
	mkdir -p dist
	SDKROOT=$(SDKROOT) cargo zigbuild --target universal2-apple-darwin --release
	cp target/universal2-apple-darwin/release/fluidsim osx/FluidSim.app/Contents/MacOS/FluidSim
	zip -r dist/fluidsim-osx-universal.zip osx/FluidSim.app
