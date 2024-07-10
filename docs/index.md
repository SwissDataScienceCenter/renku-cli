# Renku CLI

This is the documentation for the command line interface to the Renku
platform.


## Installation

The binary name for the renku-cli is `rnk`.

### Manual Download

You can download the binary for your platform from the [release
page](https://github.com/SwissDataScienceCenter/renku-cli/releases/latest).

If you run on MacOS, download the `*-darwin` binary. If you run some
form of linux, try `*-amd64` or `*-aarch64`. Last for Windows use the
`*-windows` binary.

Once downloaded, you can simply execute it without any further
installation step.

#### shell completion

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

### Nix User

If you are a nix user and have flakes enabled, you can install rnk
from this repository:

```
nix profile install github:SwissDatascienceCenter/renku-cli
```

If you want to try it out without installing:
```
nix run github:SwissDatascienceCenter/renku-cli
```

When installing the package via `install` or including it into your
NixOS configuration, the shell completions are already installed.

### Debian/Ubuntu User

TODO

### Mac Homebrew

TODO

## Getting started

The renku cli accepts commands to interact with the renku platform. To
get an overview of possible commands, run the binary without any
options or adding `--help`.

``` bash renku-cli
rnk --help
```
