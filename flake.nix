{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };
  outputs = flakes:
    let
      system = "x86_64-linux";
      pkgs = import flakes.nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = [
          pkgs.bashInteractive
          pkgs.rustup
          pkgs.mypy
          (pkgs.python3.withPackages (ps: [
            ps.matplotlib
            ps.numpy
          ]))
        ];
      };
    };
}

