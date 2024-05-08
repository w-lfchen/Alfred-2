{
  description = "A very basic flake";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          cargo
          clippy
          rust-analyzer
          rustc
          rustfmt
          sccache

          pkg-config
          openssl
        ];
        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        shellHook = ''
          export RUSTC_WRAPPER=sccache
        '';
      };
      # currently broken due to how the token is handeled
      #packages.${system} = {
      #  alfred-2 = pkgs.rustPlatform.buildRustPackage {
      #    name = "alfred-2";
      #    src = ./.;
      #    cargoLock.lockFile = ./Cargo.lock;
      #    buildInputs = with pkgs; [ openssl ];
      #    nativeBuildInputs = with pkgs; [ pkg-config ];
      #  };
      #  default = self.packages.${system}.alfred-2;
      #};
      formatter.${system} = pkgs.nixfmt-rfc-style;
    };
}
