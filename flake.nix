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
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
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
            llvmPackages_latest.libllvm
          ];

          RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";
          RUSTFLAGS = "-Clink-arg=-fuse-ld=mold";

          LLVM_SYS_201_PREFIX = "${pkgs.llvmPackages_latest.llvm.dev}";
        };
      }
    );
}
