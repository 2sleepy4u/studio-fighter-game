{ pkgs ? import <nixpkgs> {} }:

/*
based on
https://discourse.nixos.org/t/how-can-i-set-up-my-rust-programming-environment/4501/9
*/
let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  rustVersion = "latest";
  #rustVersion = "1.62.0";
  rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
    extensions = [
      "rust-src" # for rust-analyzer
      "rust-analyzer"
    ];
  };
in
pkgs.mkShell rec {
  buildInputs = [
    rust
  ] ++ (with pkgs; [
    pkg-config
	amdvlk
    vulkan-tools
    binutils
    libGL
    mesa
    udev alsa-lib vulkan-loader libxkbcommon wayland
	gdb

    # other dependencies
    #gtk3
    #wrapGAppsHook
  ]);
  RUST_BACKTRACE = 1;
  LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath buildInputs;
}
