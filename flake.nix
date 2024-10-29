{
  description = "reimplementation of biomorph-evolve in rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
      with pkgs;
      {
        devShells.default = mkShell rec {

          packages = [
            cargo-bloat
            upx
          ];

          buildInputs = [
            # Rust
            rust-bin.stable.latest.default
            trunk

            # misc. libraries
            pkg-config
            alsa-lib
            udev

            # GUI libs
            libxkbcommon
            libGL
            fontconfig

            vulkan-loader

            wayland
          ];

          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
        };
      }
    );
}
