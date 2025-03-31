{
  description = "rustcc";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
          ];
        };
      in {
        devShell = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [
            pkg-config
            libxml2
            zlib
          ];

          RUST_SRC_PATH = "${pkgs.rust-bin.stable.latest.default.override {
            extensions = ["rust-src"];
          }}/lib/rustlib/src/rust/library";

          buildInputs = with pkgs; [
            # Tooling
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = ["rust-src" "cargo" "rustc"];
            })
            mold
            sccache
            cargo-fuzz
            cargo-mutants
            cargo-tarpaulin

            # LLVM
            llvmPackages_19.libllvm
          ];

          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath nativeBuildInputs}:${pkgs.stdenv.cc.cc.lib.outPath}/lib:$LD_LIBRARY_PATH";

          RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";
          RUSTFLAGS = "-Clink-arg=-fuse-ld=mold";

          LLVM_SYS_191_PREFIX = "${pkgs.llvmPackages_19.llvm.dev}";
        };
      }
    );
}
