.PHONY: all win

all:
	cargo build --release

win:
	cargo xwin build --target x86_64-pc-windows-msvc --release
