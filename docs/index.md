# Renku CLI

This is the documentation for `renku-cli` the command line interface
to the Renku platform.


## Installation

The binary name for the renku-cli is `renku-cli`.

### Manual Download

You can download the binary for your platform from the [release
page](https://github.com/SwissDataScienceCenter/renku-cli/releases/latest).

If you run on MacOS, download the `*-darwin` binary. If you run some
form of linux, try `*-amd64` or `*-aarch64`. Last for Windows use the
`*-windows` binary.

### Nix User

If you are a nix user and have flakes enabled, you can install renku-cli
from this repository:

```
nix profile install github:SwissDatascienceCenter/renku-cli
```

If you want to try it out without installing:
```
nix run github:SwissDatascienceCenter/renku-cli
```

### Debian/Ubuntu User

TODO

### Mac Homebrew

TODO

## Getting started

The renku cli accepts commands to interact with the renku platform. To
get an overview of possible commands, run the binary without any
options or adding `--help`.

``` bash renku-cli
renku-cli --help
```
