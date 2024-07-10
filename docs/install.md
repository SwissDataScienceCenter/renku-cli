# Install

The binary name for the renku-cli is `rnk`.

## Manual

You can download the binary for your platform from the [release
page](https://github.com/SwissDataScienceCenter/renku-cli/releases/latest).

If you run on MacOS, download the `*-darwin` binary. If you run some
form of linux, try `*-amd64` or `*-aarch64`. Last for Windows use the
`*-windows` binary.

Once downloaded, you can simply execute it without any further
installation step.

## Nix and NixOS

### Nix

If you are a [nix](https://nixos.org/nix) user and have flakes
enabled, you can install rnk from this repository:

```
nix profile install github:SwissDatascienceCenter/renku-cli/<version>
```

If `/<version>` is omitted, it will install the current development
version right off the `main` branch.

If you want to try it out without installing:
```
nix run github:SwissDatascienceCenter/renku-cli
```

When installing the package via `install` or including it into your
NixOS configuration, the shell completions are already installed.

### NixOS

When you are a NixOS user, you can include the flake and select the
provided package in your `configuration.nix`. Here is an example:

``` nix
{
  description = "Example rnk for NixOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rnk.url = "github:SwissDataScienceCenter/renku-cli/<version>";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    rnk,
    ...
  }: let
    system = "x86_64-linux";

    overlays = [
      # ... more of your overlays
      rnk.overlays.default
    ];

    pkgsBySystem = system:
      import nixpkgs {
        inherit system;
        inherit overlays;
      };

    modules = [
      {
        # build as a container to remove the need for specific
        # filesystems
        boot.isContainer = true;
        system.stateVersion = "24.05";
        # set pkgs to be the one with overlays applied
        nixpkgs.pkgs = pkgsBySystem system;
      }

      # select rnk as a package in your modules
      ({pkgs, ...}: {environment.systemPackages = [pkgs.rnk];})
    ];
  in {
    nixosConfigurations.mymachine = nixpkgs.lib.nixosSystem {
      inherit system modules;
      specialArgs = inputs;
    };
  };
}
```

The above configuration can be build into a NixOS system:
``` bash
nix build .#nixosConfigurations.mymachine.config.system.build.toplevel
```

If you are not using flakes, you can import the derivation from
`default.nix`.

You can omit `/<version>` in the input url, if you want to install
latest development version, otherwise replace `<version>` with an
existing tag.

## Linux and Mac User

The convenient way is to use the `installer.sh` script that is
provided from this repository. It will download the correct binary
from the release page and put it in `/usr/local/bin` on your system.
It requires `curl` and `sudo` to copy the binary to `/usr/local/bin`.

```
curl -sfSL https://raw.githubusercontent.com/SwissDataScienceCenter/renku-cli/main/install.sh | bash
```

If you want to uninstall, simply remove the `/usr/local/bin/rnk` file.


## Shell Completion

For convenience, the cli tool can generate completion commands for
several shells. You can use it for inclusion in your `.bashrc` or
similar setups.

For example:

``` bash rnk:silent
rnk shell-completion --shell bash
```

will generate the completions for bash. These have to be "sourced"
into into your current shell:

``` bash
eval "$(rnk shell-completion --shell bash)"
```

Add this line to your `.bashrc` to have these completions available
when you enter bash.

With this enabled, when you type `rnk <tab>` you will be presented
with possible options, that are narrowed down the more letters you
type.
