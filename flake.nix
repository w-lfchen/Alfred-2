{
  description = "alfred-2 dev flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      fenix,
      nixpkgs,
    }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          pkgs.openssl.dev
          pkgs.pkg-config
          (fenix.packages.${system}.stable.withComponents [
            "cargo"
            "clippy"
            "rust-analyzer"
            "rust-src"
            "rustc"
            "rustfmt"
          ])
        ];
      };
      packages.${system} =
        let
          toolchain = fenix.packages.${system}.minimal.toolchain;
          pkgs = nixpkgs.legacyPackages.${system};
          rustPlatform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
        in
        {
          alfred-2 = rustPlatform.buildRustPackage {
            name = "alfred-2";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes =
                let
                  typst-hash = "sha256-XH/0JfJQFYsSVLGoUH4OnqTWWoOrmDw5MFZrRwLJuBw=";
                in
                {
                  "typst-0.14.2" = typst-hash;
                  "typst-assets-0.14.2" = "sha256-cdGuHtXc1vwL84jLT96dD1M4kbfO5hybMSh9cvedqKw=";
                  "typst-kit-0.14.2" = typst-hash;
                  "typst-layout-0.14.2" = typst-hash;
                  "typst-render-0.14.2" = typst-hash;
                };
            };
          };
          default = self.packages.${system}.alfred-2;
        };
      formatter.${system} = pkgs.nixfmt;
    };
}
