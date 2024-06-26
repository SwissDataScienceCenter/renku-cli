# Contributing

## Setting up dev environment

### nix

The easiest way to setup everything is to install
[nix](https://nixos.org/nix).

Then run `nix develop` in the source root. Alternatively, using
[direnv](https://direnv.net) run `direnv allow`. Both will drop you in
a shell with everything setup.

Editors usually support `direnv`, so this can be used to integrate it
into your favorite editor.

### Manual

Requirements:
- recent rust compiler and cargo (see
  [rustup](https://rust-lang.github.io/rustup/))
- mold (optionally)

### Mold

The nix setup enables [mold](https://github.com/rui314/mold) for
faster compiling (linking). It can save quite some time when compiling
frequently. Here is an example of a clean build (on a very early small
version of this tool):

with mold:
```
Executed in   37.00 secs    fish           external
   usr time  322.81 secs    0.00 micros  322.81 secs
   sys time   21.67 secs  768.00 micros   21.67 secs
```

without mold:
```
Executed in   59.34 secs    fish           external
   usr time  624.70 secs  360.00 micros  624.70 secs
   sys time   40.01 secs  230.00 micros   40.01 secs
```

However, if you experience problems, you can disable mold by unsetting
these environment variables:

```
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS
```

Likewise, when not using nix, set these env vars to enable mold (see
`flake.nix`).

## Compile & Run

Run:
```
cargo build
```

to compile the sources. You can run the cli from sources using

```
cargo run -- <options to the cli tool here>
```

Run it without any options after the `--` to see a quick help.

## Nix Setup Description

The `flake.nix` defines how to build and test this package by
utilising the existing cargo build. It uses
[crane](https://github.com/ipetkov/crane) for integrating the cargo
build into nix.

For setting up the rust toolchain,
[fenix](https://github.com/nix-community/fenix) is used.

The `commonArgs` in `flake.nix` defines all necessary dependencies to
build the binary. Additionally, the `devShells.default` attributes
define what is available in the development shell (that is enabled by
either `nix develop` or automatically entered when `direnv allow` has
been run).

To build the binary, run:
```
nix build
```

This builds the final package (with `--release` passed to cargo
build). It will first build all dependencies, so rebuilding after a
change is quicker when only the sources need to be re-compiled.

For running all the tests and checks, run
```
nix flake check
```

For auto-formatting the nix files run
```
nix fmt
```

The formatter used,
[alejandra](https://github.com/kamadorueda/alejandra) is also defined
in `flake.nix`.

### Dev Shells

There can be multiple development shells defined, where the `default`
attribute is used when nothing is explicitely defined.

Creating a different dev shell, simply add another attribute and enter it using:
```
nix develop .#<your-devshell-attribute>
```

Alternatively, edit `.envrc` to read `use flake .#<your-devshell-attribute>`.


## Dev Cookbook

### Adding a new (sub) command

1. Implement the command analogous to others in `src/cli/cmd/my_sub.rs`
2. Add a new variant to `SubCommand` enum in `opts.rs`
3. Follow the compile errors:
   - in `cli.rs` run the command, most likely analogous to the
     existing ones
   - in `cmd.rs` add another `From` impl for the error (if necessary)
