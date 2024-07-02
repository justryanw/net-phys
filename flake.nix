{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        toolchian = with fenix.packages.${system}; combine [
          stable.rustc
          stable.cargo
          stable.rustfmt
          targets.x86_64-pc-windows-gnu.stable.rust-std
        ];

        craneLib = ((crane.mkLib nixpkgs.legacyPackages.${system}).overrideToolchain toolchian);

        buildDeps = (with pkgs; [
          pkg-config
          makeWrapper
          clang
          mold
        ]);

        runtimeDeps = pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
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

        commonArgs = pname: {
          inherit pname;
          version = "1.0.0";
          src = ./.;
          cargoExtraArgs = "--package=${pname}";
        };

        linux = pname: (commonArgs pname) // {
          nativeBuildInputs = buildDeps;
          buildInputs = runtimeDeps;

          postInstall = ''
            wrapProgram $out/bin/${pname} \
              --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath runtimeDeps} \
              --prefix XCURSOR_THEME : "Adwaita"
            cp -a assets $out/bin
          '';
        };

        windows = pname: (commonArgs pname) // {
          strictDeps = true;
          doCheck = false;

          nativeBuildInputs = buildDeps ++ [ pkgs.zip ];

          CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";

          # Fixes issues related to libring
          TARGET_CC = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/${pkgs.pkgsCross.mingwW64.stdenv.cc.targetPrefix}cc";

          # Fixes issues related to openssl
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";

          depsBuildBuild = with pkgs; [
            pkgsCross.mingwW64.stdenv.cc
            pkgsCross.mingwW64.windows.pthreads
          ];

          postInstall = ''
            mkdir -p $out/bin/assets
            cp -a assets $out/bin
            cd $out/bin
            zip -r ../${pname}.zip *
            cd ..
            rm -r bin
          '';
        };

        cargoArtifacts = craneLib.buildDepsOnly (linux "client");

        client = craneLib.buildPackage (linux "client") // { inherit cargoArtifacts; };
        server = craneLib.buildPackage (linux "server") // { inherit cargoArtifacts; };

        windows-client = craneLib.buildPackage (windows "client");
        windows-server = craneLib.buildPackage (windows "server");
      in
      {
        checks = {
          inherit client server;
        };

        packages = {
          inherit client server windows-client windows-server;
          default = client;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath runtimeDeps}";
          XCURSOR_THEME = "Adwaita";
        };
      });
}
