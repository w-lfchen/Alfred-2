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
          (fenix.packages.${system}.stable.withComponents [
            "cargo"
            "clippy"
            "rust-analyzer"
            "rust-src"
            "rustc"
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
            cargoLock.lockFile = ./Cargo.lock;
          };
          default = self.packages.${system}.alfred-2;
        };
      formatter.${system} = pkgs.nixfmt;
    };
}
