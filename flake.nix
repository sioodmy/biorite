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
    fenix,
    nixpkgs,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        toolchain = with fenix.packages.${system};
          combine [
            minimal.rustc
            minimal.cargo
            complete.clippy-preview
            complete.rustfmt-preview
            complete.rust-analyzer-preview
            targets.x86_64-unknown-linux-musl.latest.rust-std
            targets.x86_64-pc-windows-gnu.latest.rust-std
          ];

        naersk' = naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        };

        build-deps = with pkgs; [
          llvmPackages.bintools
          mold
          lld
          clang
          pkg-config
          makeWrapper
          libxkbcommon
          wayland
          jemalloc
        ];
        runtime-deps = with pkgs; [
          # TODO: remove unused libs
          alsa-lib
          udev
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          xorg.libxcb
          libGL
          freetype
          fontconfig
          xorg.xorgproto
          xorg.libXt
          xorg.libXft
          xorg.libXext
          xorg.libSM
          xorg.libICE
          vulkan-loader
          vulkan-headers
          vulkan-validation-layers
          vulkan-tools
          libxkbcommon
          wayland
          glfw-wayland
        ];
      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          buildInputs = runtime-deps;
          nativeBuildInputs = build-deps;
          src = ./.;
          BEVY_ASSET_PATH = ./assets;
          cargoBuildOptions = x: x ++ ["--no-default-features"];
          overrideMain = attrs: {
            fixupPhase = ''
              wrapProgram $out/bin/biorite\
                --prefix LD_LIBRARY_PATH : ${
                pkgs.lib.makeLibraryPath runtime-deps
              } \
                --set CARGO_MANIFEST_DIR $out/share/biorite
            '';
          };
        };

        apps.biorite = flake-utils.lib.mkApp {drv = defaultPackage;};
        apps.default = apps.biorite;

        # For `nix develop`:
        devShell = with pkgs; let
          # https://discourse.nixos.org/t/using-mold-as-linker-prevents-libraries-from-being-found/18530/4
          bintools-wrapper = "${nixpkgs}/pkgs/build-support/bintools-wrapper";
          mold' = symlinkJoin {
            name = "mold";
            paths = [mold];
            nativeBuildInputs = [pkgs.makeWrapper];
            suffixSalt = lib.replaceStrings ["-" "."] ["_" "_"] targetPlatform.config;
            postBuild = ''
              for bin in ${mold}/bin/*; do
                rm $out/bin/"$(basename "$bin")"

                export prog="$bin"
                substituteAll "${bintools-wrapper}/ld-wrapper.sh" $out/bin/"$(basename "$bin")"
                chmod +x $out/bin/"$(basename "$bin")"

                mkdir -p $out/nix-support
                substituteAll "${bintools-wrapper}/add-flags.sh" $out/nix-support/add-flags.sh
                substituteAll "${bintools-wrapper}/add-hardening.sh" $out/nix-support/add-hardening.sh
                substituteAll "${bintools-wrapper}/../wrapper-common/utils.bash" $out/nix-support/utils.bash
              done
            '';
          };
        in
          mkShell.override {stdenv = gcc12Stdenv;} {
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            NIX_CFLAGS_LINK = "-fuse-ld=mold";
            RUST_LOG = "info,wgpu_core=warn,wgpu_hal=off,rechannel=warn,biorite=debug";
            shellHook = ''
              mkdir -p world/regions
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath runtime-deps}"
            '';
            buildInputs =
              [toolchain pciutils]
              ++ runtime-deps
              ++ build-deps;
          };
      }
    );
}
