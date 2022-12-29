{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    flake-utils,
    naersk,
    nixpkgs,
    fenix,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        toolchain = fenix.packages.${system}.latest.toolchain;

        naersk' = naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        };

        build-deps = with pkgs; [
          llvmPackages.bintools
          lld
          clang
          pkg-config
          makeWrapper
          libxkbcommon
          wayland
        ];
        runtime-deps = with pkgs; [
          alsa-lib
          udev
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          xorg.libxcb
          libGL
          vulkan-loader
          vulkan-headers
          libxkbcommon
          wayland
          glfw-wayland
          zstd
        ];
      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          buildInputs = runtime-deps;
          nativeBuildInputs = build-deps;
          src = ./.;
          overrideMain = attrs: {
            fixupPhase = ''
              wrapProgram $out/bin/voxelorite\
                --prefix LD_LIBRARY_PATH : ${
                pkgs.lib.makeLibraryPath runtime-deps
              } \
                --set CARGO_MANIFEST_DIR $out/share/voxelorite
                mkdir -p $out/share/voxelorite
                cp -a assets $out/share/voxelorite
            '';
          };
        };

        apps.voxelorite = flake-utils.lib.mkApp {drv = defaultPackage;};
        apps.default = apps.voxelorite;

        # For `nix develop`:
        devShell = with pkgs;
          mkShell {
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtime-deps;
            RUST_BACKTRACE = 1;
            buildInputs =
              [
                toolchain
                rustc
                cargo
                clippy
                rustfmt
                rust-analyzer
              ]
              ++ runtime-deps
              ++ build-deps;
          };
      }
    );
}
