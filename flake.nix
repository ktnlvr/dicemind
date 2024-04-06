# todo: format me, help me, save me, kill me pls, FIX MEEEEEE
{
  inputs = {
	nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
	rust-overlay.url = "github:oxalica/rust-overlay";
	flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
  flake-utils.lib.eachDefaultSystem(system:
	let
	  overlays = [ (import rust-overlay) ];
	  overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
	  rustVersion = overrides.toolchain.channel;
	  rust = overrides.toolchain.channel.${rustVersion}.default.override {
		extensions = [
		  "rust-src"
		  "rust-analyzer"
		];
	  };
	  pkgs = import nixpkgs {
		inherit system overlays;
	  };
	in
	{ devShells.default = pkgs.mkShell rec {
		nativeBuildInputs = with pkgs; [];
		buildInputs = with pkgs; [
		  clang
		  # Replace llvmPackages with llvmPackages_X, where X is the latest LLVM version (at the time of writing, 16)
		  llvmPackages.bintools
		  rustup
		];
		RUST_BACKTRACE = 1;
		RUSTC_VERSION = rustVersion;
		# https://github.com/rust-lang/rust-bindgen#environment-variables
		LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
		shellHook = ''
		  export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
		  export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
		  '';
		RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') []);
		# Add glibc, clang, glib, and other headers to bindgen search path
		BINDGEN_EXTRA_CLANG_ARGS =
		# Includes normal include path
		(builtins.map (a: ''-I"${a}/include"'') [
		  # add dev libraries here (e.g. pkgs.libvmi.dev)
		  pkgs.glibc.dev
		])
		# Includes with special directory paths
		++ [
		  ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
		  ''-I"${pkgs.glib.dev}/include/glib-2.0"''
		  ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
		];
	  };
	}
  );
}
