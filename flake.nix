{
  description = "A blazing fast fetch tool written in rust.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = self.lib.makePackage pkgs { };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
            pkg-config
            rust-analyzer
          ];
        };
      }
    )
    // {
      lib.makePackage =
        pkgs: userSettings:
        let
          lib = pkgs.lib;

          defaultModules = {
            title = {
              index = 1;
              enabled = true;
              key = "";
            };
            os = {
              index = 2;
              enabled = true;
              key = "OS";
            };
            host = {
              index = 3;
              enabled = true;
              key = "Host";
            };
            kernel = {
              index = 4;
              enabled = true;
              key = "Kernel";
            };
            uptime = {
              index = 5;
              enabled = true;
              key = "Uptime";
            };
            shell = {
              index = 6;
              enabled = true;
              key = "Shell";
            };
            de = {
              index = 7;
              enabled = true;
              key = "DE";
            };
            theme = {
              index = 8;
              enabled = true;
              key = "Theme";
            };
            cursor = {
              index = 9;
              enabled = true;
              key = "Cursor";
            };
            cpu = {
              index = 10;
              enabled = true;
              key = "CPU";
            };
            gpu = {
              index = 11;
              enabled = true;
              key = "GPU";
            };
            ram = {
              index = 12;
              enabled = true;
              key = "Memory";
            };
            swap = {
              index = 13;
              enabled = true;
              key = "Swap";
            };
            locale = {
              index = 14;
              enabled = true;
              key = "Locale";
            };
            standard_palette = {
              index = 15;
              enabled = true;
              key = "";
            };
            bright_palette = {
              index = 16;
              enabled = true;
              key = "";
            };
          };

          mergedModules = lib.recursiveUpdate defaultModules (userSettings.modules or { });

          finalModuleList =
            let
              listWithIds = lib.mapAttrsToList (name: val: val // { id = name; }) mergedModules;
              sorted = builtins.sort (a: b: a.index < b.index) listWithIds;
            in
            map (m: removeAttrs m [ "index" ]) sorted;

          finalConfig =
            (lib.recursiveUpdate {
              colors = {
                title = "#b19cd9";
                keys = "#b19cd9";
                separator = "#ff6961";
                values = "#eec1cb";
              };
              logo = {
                enabled = true;
                include = [
                  {
                    id = "nixos";
                    colors = [
                      "#967ce2"
                      "#8951c1"
                    ];
                  }
                ];
              };
            } userSettings)
            // {
              modules.general = finalModuleList;
            };

          configFile = (pkgs.formats.yaml { }).generate "config.yaml" finalConfig;
        in
        pkgs.rustPlatform.buildRustPackage {
          pname = "qfetch";
          version = "0.1.2";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          postPatch = "cp ${configFile} config.yaml";
        };

      nixosModules.default =
        {
          config,
          pkgs,
          lib,
          ...
        }:
        {
          options.qfetch.settings = lib.mkOption {
            type = lib.types.attrs;
            default = { };
          };
          config.environment.systemPackages = [
            (self.lib.makePackage pkgs config.qfetch.settings)
          ];
        };
    };
}
