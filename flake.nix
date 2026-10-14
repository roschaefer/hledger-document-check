{
  description = "Check hledger transactions against documents on disk";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        lib = pkgs.lib;

        # Scrut is not in nixpkgs yet, so use its upstream release binaries.
        scrut =
          let
            version = "0.4.3";
            platforms = {
              x86_64-linux = { name = "linux-x86_64"; hash = "sha256-au+wv/KbRQ+9GwQIPu/q5DiNbJMC4x0aJLeqnt0DuzU="; };
              aarch64-linux = { name = "linux-aarch64"; hash = "sha256-pnEX9DINEpx9cHOl5mfbrSXnqD8xSApjMQglD+QGndU="; };
              x86_64-darwin = { name = "macos-x86_64"; hash = "sha256-JYlzAFESfhFOIBSRblHn6s6wH1nqe/sqpZvzbI+izJ4="; };
              aarch64-darwin = { name = "macos-aarch64"; hash = "sha256-PAvI1FX92zguHpCZ9fSk4LjpJbGfpYjCF2vARjg9mrw="; };
            };
            platform = platforms.${system};
          in
          pkgs.stdenv.mkDerivation {
            pname = "scrut";
            inherit version;
            src = pkgs.fetchurl {
              url = "https://github.com/facebookincubator/scrut/releases/download/v${version}/scrut-v${version}-${platform.name}.tar.gz";
              inherit (platform) hash;
            };
            sourceRoot = "scrut-${platform.name}";
            nativeBuildInputs = lib.optionals pkgs.stdenv.isLinux [ pkgs.autoPatchelfHook ];
            buildInputs = lib.optionals pkgs.stdenv.isLinux [ pkgs.stdenv.cc.cc.lib ];
            installPhase = "install -Dm755 scrut $out/bin/scrut";
            meta.platforms = builtins.attrNames platforms;
          };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.git
            pkgs.rustc
            pkgs.cargo
            pkgs.rustfmt
            pkgs.clippy
            pkgs.cargo-audit
            pkgs.cargo-machete
            pkgs.hledger
            pkgs.just
            scrut
          ];

          shellHook = ''
            repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            export PATH="$repo_root/hledger-document-check/target/debug:$PATH"
          '';
        };
      });
}
