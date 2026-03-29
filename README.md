# qfetch
A blazing fast, highly configurable and lightweight fetch tool written in Rust.

## Installation
### cargo
```bash
CONFIG_FILE_PATH=/path/to/file cargo install qfetch-cli
```
### Manual Build
```bash
cargo build --release
```
You can optionally use CONFIG_FILE_PATH=/path/to/file before the build command if you want to use a custom path for it.

### NixOS
Add qfetch to your nix flake inputs
```nix
inputs.qfetch.url = "github:quixaq/qfetch";
```
Add the module to nixosSystem
```nix
outputs = { nixpkgs, qfetch, ... }: {
  nixosConfigurations.<hostname> = nixpkgs.lib.nixosSystem {
    modules = [
      ./configuration.nix
      qfetch.nixosModules.default
    ];
  };
};
```

## Configuration
### General
Remember to use full hex codes in colors since expanding them is not implemented.
For `logo.include`, the first logo will be used as fallback, also remember that every logo you include will be directly included in the binary so it may increase execution time. You can see the available logos in the `logo` dir.
### cargo
Get the default `config.yaml` file from the repository and make changes there and point the CONFIG_FILE_PATH env variable to it.
The modules are ordered in the way you order them in the config.
### Manual Build
Modify the `config.yaml` file in the project dir and rebuild.
The modules are ordered in the way you order them in the config.
### NixOS
You can config directly in your `configuration.nix`.
You'll need to set the index manually due to the way the config is handled. You can use floats and the numbers don't have to be in order since it's sorted before being injected into the project dir.

Example config:
```nix
qfetch.settings = {
  modules = {
    os = { enabled = true; key = "Distro"; }
    kernel.enabled = false;
    gpu.key = "Graphics";
  };
  
  colors = {
    title = "#ffffff";
  };
  
  logo = {
    enabled = true;
    include = [
      { id = "nixos" colors = [ "#ffafcb" "#123456" ]; }
    ];
  };
};
```

## Benchmarks
qfetch with all modules enabled:
```bash
> hyperfine -N --warmup 2500 qfetch
Benchmark 1: ./qfetch/target/release/qfetch
  Time (mean ± σ):       1.5 ms ±   0.1 ms    [User: 1.1 ms, System: 0.4 ms]
  Range (min … max):     1.3 ms …   1.9 ms    2039 runs
```

fastfetch will the same modules enabled:
```bash
> hyperfine -N --warmup 2500 fastfetch
Benchmark 1: fastfetch
  Time (mean ± σ):      48.4 ms ±   0.8 ms    [User: 23.7 ms, System: 24.3 ms]
  Range (min … max):    47.6 ms …  51.6 ms    62 runs
```
