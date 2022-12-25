{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = {
    self,
    flake-utils,
    naersk,
    nixpkgs,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};

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

        # For `nix develop`:
        devShell = with pkgs;
          mkShell {
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            buildInputs =
              [rustc cargo rustPackages.clippy]
              ++ runtime-deps
              ++ build-deps;
          };
      }
    );
}
