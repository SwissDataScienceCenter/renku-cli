# preliminary docs

# Goal

- [x] outline of cli app
- [x] some tests
- [x] github ci workflows:
  - [x] for publishing (what/how exactly? assume to upload a binary to
        the github release page)
  - [x] for testing an all platforms
- [ ] add tokio and async all the things
- [ ] nice to have: clone a project / repo
  - the notion document talks about clonig a project first, then
    another sentence mentions cloning repositories
  - which is it? if project, how is cloning multiple repos supposed to
    work? and is there anything else than `git clone` to be done?
- [ ] think better what is `pub` and what is not, thinking about
      providing a rust library alongside the cli maybe
- [ ] publish musl builds with dynamically linking to openssl (perhaps
      using this https://github.com/rust-cross/rust-musl-cross)
- [ ] figure how to cross compile using nix to make it better reproducible

## Design/Outline

- it currently is a single application crate
  - it might be better to start with a multi project?
  - we might want to add tooling, like creating man pages etc
  - it could also be consumed as a library 
- the cli uses clap as the argument parsing library, which comes
  batteries-included provding a lot of convenience features out of the
  box
- the main command does nothing by itself, but will always accept a sub-command
- the main accepts only a few flags that provide data common to all sub-commands
- all sub commands are write to stdout/stderr at some point, which is captured in the trait `Cmd`
- when writing the resulting values, they should implement `Sink` in
  order to have a consistent output - currently json for machine
  reading and "default" for humans (using the `Display` trait)
- sub commands can themselves accept sub commands (simply recursing),
  the `project` command exists as an example
- serde macros are used to provide JSON de/serializer
- all commands should support two types of output: json and "human
  readable", so the cli can be used in scripts and as a user
- reqwest is used to run http requests
  - the module `httpclient` implements a higher level http client
    adopted to renkus remote api
  - currently, I opted for the synchronous variant, because:
    - it results in simpler code
    - right now there is no command doing lots of (io bound)
      operations that would result in significant performance gain.
      When we need to run many many requests/doing lots of io things,
      then we should probably introduce tokio and use the async
      variant of reqwest
- errors are encoded per module (currently transparent) and can be
  amended with context using Snafu, creating a hierarchy
- the `assert_cmd` crate is used to provide integration tests
- the `env_log` library is used for logging, it prints to stderr by
  default which is useful to provide extra information the user can
  opt-in via the `-v(vv)` flags

## Adding a new sub command

1. Implement the command analogous to others in `src/cli/cmd/my_sub.rs`
2. Add a new variant to `SubCommand` enum in `opts.rs`
3. Follow the compile errors:
   - in `cli.rs` run the command
   - in `cmd.rs` add another `From` impl for the error (if necessary)

## git 

There are the following ways to do git operations:

- git2-rs: requires either to link to an exisitng libgit2 or include
  the library in the binary
  - pro: uses what git uses
  - con: may involve installing a dependency with the cli, api may be
    influenced by the C one
- gitoxide: a library entirely implemented in rust 
  - pro: looks fancy, rust api is probably nicer to work with
  - con: not what git uses…, not all features supported
- use the existing git binary
  - pro: smaller binary
  - con: requires managing external processes (which should be easy in
    rust, though), requires git as dependency
