{
  description = "Development environment for cosense-md";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.cargo
              pkgs.rustc
              pkgs.clippy
              pkgs.rustfmt
              # Default wasm32-unknown-unknown linker of the nixpkgs rustc.
              pkgs.lld
              # The PyO3 build script runs it.
              pkgs.python3
              pkgs.uv
              # Must match the wasm-bindgen crate version.
              pkgs.wasm-bindgen-cli_0_2_130
            ];
          };
        }
      );
    };
}
