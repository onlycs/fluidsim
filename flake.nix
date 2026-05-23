{
  description = "Fluid Simulation Development";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        osxsdk = fetchTarball {
          url = "https://github.com/joseluisq/macosx-sdks/releases/download/11.3/MacOSX11.3.sdk.tar.xz";
          sha256 = "173vy527rsgznqwj9dfrr8lpggvbbhh5hfh543n5c63srb4fl26d";
        };

        libraries = with pkgs; [
          pkg-config

          wayland
          libxkbcommon

          libGL
          vulkan-loader
          vulkan-headers

          gcc.cc.lib
        ];

        inputs = with pkgs; [
          rustToolchain
          sccache
          mold
          wasm-pack

          clang
          lld
          cargo-xwin

          zig
          cargo-zigbuild
          zip
          osxsdk

          nil
          nixd
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = inputs ++ libraries;
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libraries;
          PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" libraries;
          SDKROOT = osxsdk;
        };
      }
    );
}
