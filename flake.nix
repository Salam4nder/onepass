{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rust = pkgs.rust-bin.stable.latest.default;
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.nixfmt
            rust
          ];

          shellHook = ''
            echo Entered the development environment.
            export RUST_LOG=debug
            alias cr='cargo r'
            alias ct='cargo t'
            alias ctn='cargo t -- --nocapture'
            alias cc='cargo c'
            export RUST_BACKTRACE=1
            export PS1="$PS1[nix-shell] "
          '';
        };
      }
    );
}
