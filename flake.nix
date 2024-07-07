{
  nixConfig = {
    extra-trusted-public-keys = "justryanw.cachix.org-1:oan1YuatPBqGNFEflzCmB+iwLPtzq1S1LivN3hUzu60=";
    extra-substituters = "https://justryanw.cachix.org";
    allow-import-from-derivation = true;
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    # TODO remove when 1.79 is merged into nixpkgs
    nixpkgs-for-rust.url = "github:NixOS/nixpkgs/staging";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    crate2nix = {
      url = "github:nix-community/crate2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ { self, flake-parts, crate2nix, ... }: flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];

    perSystem = { system, pkgs, lib, ... }:
      let
        buildInputs = (with pkgs; [
          libxkbcommon
          alsa-lib
          udev
          vulkan-loader
          wayland
        ] ++ (with xorg; [
          libXcursor
          libXrandr
          libXi
          libX11
        ]));

        porjectName = "net-phys";

        workspaceMemberOverride = workspaceMember: attrs: {
          name = "${porjectName}-${workspaceMember}-${attrs.version}";
        };

        cargoNix = pkgs.callPackage
          (crate2nix.tools.${system}.generatedCargoNix {
            name = porjectName;
            src = ./.;
          })
          {
            defaultCrateOverrides = pkgs.defaultCrateOverrides // {
              wayland-sys = atts: {
                nativeBuildInputs = with pkgs; [ pkg-config ];
                buildInputs = with pkgs; [ wayland ];
              };

              client = attrs: ((workspaceMemberOverride "client" attrs) // {
                nativeBuildInputs = [ pkgs.makeWrapper ];

                postInstall = ''
                  wrapProgram $out/bin/client \
                    --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath buildInputs} \
                    --prefix XCURSOR_THEME : "Adwaita"
                  mkdir -p $out/bin/assets
                  cp -a assets $out/bin
                '';
              });

              server = (workspaceMemberOverride "server");
            };
          };

        build-workspace-member = workspaceMemeber: cargoNix.workspaceMembers.${workspaceMemeber}.build;

        pkgs-for-rust = inputs.nixpkgs-for-rust.legacyPackages.${system};

        overlays = [
          (final: prev: {
            rustc = pkgs-for-rust.rustc;
            cargo = pkgs-for-rust.cargo;
          })
        ];
      in
      {
        _module.args.pkgs = import inputs.nixpkgs { inherit system overlays; config = { }; };

        packages = {
          client = build-workspace-member "client";
          server = build-workspace-member "server";
          default = self.packages.${system}.client;
        };

        devShells.default = pkgs.mkShell {
          inherit buildInputs;

          nativeBuildInputs = with pkgs; [
            cargo
            rustc
            pkg-config
            rustfmt
            clang
            mold
          ];

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
          XCURSOR_THEME = "Adwaita";
        };
      };
  };
}
