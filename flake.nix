{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    fenix,
    flake-utils,
    advisory-db,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      inherit (pkgs) lib;

      craneLib = crane.mkLib pkgs;
      src = craneLib.cleanCargoSource ./.;
      docSrc = ./docs;

      fenixToolChain = fenix.packages.${system}.complete;

      # Common arguments can be set here to avoid repeating them later
      commonArgs = {
        inherit src;
        strictDeps = true;

        nativeBuildInputs = [
          pkgs.pkg-config
        ];
        buildInputs =
          [
            # Add additional build inputs here
            pkgs.openssl
            pkgs.installShellFiles
          ]
          ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
          ];

        # Additional environment variables can be set directly
        # MY_CUSTOM_VAR = "some value";
      };

      craneLibLLvmTools =
        craneLib.overrideToolchain
        (fenixToolChain.withComponents [
          "cargo"
          "llvm-tools"
          "rustc"
        ]);

      # Build *just* the cargo dependencies, so we can reuse
      # all of that work (e.g. via cachix) when running in CI
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      # Build the actual crate itself, reusing the dependency
      # artifacts from above.
      my-crate = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
          # additional arguments to cargo for building the app, --release is already there
          cargoExtraArgs = "";
          postInstall = ''
            for shell in fish zsh bash; do
               $out/bin/rnk shell-completion --shell $shell > rnk.$shell
               installShellCompletion --$shell rnk.$shell
            done
          '';
        });
    in {
      checks = {
        # Build the crate as part of `nix flake check` for convenience
        inherit my-crate;

        # Run clippy (and deny all warnings) on the crate source,
        # again, reusing the dependency artifacts from above.
        #
        # Note that this is done as a separate derivation so that
        # we can block the CI if there are issues here, but not
        # prevent downstream consumers from building our crate by itself.
        my-crate-clippy = craneLib.cargoClippy (commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--features user-doc --all-targets -- --deny warnings";
          });

        my-crate-doc = craneLib.cargoDoc (commonArgs
          // {
            inherit cargoArtifacts;
          });

        my-user-docs = craneLib.mkCargoDerivation (commonArgs
          // {
            inherit cargoArtifacts;
            pnameSuffix = "-userdocs";
            buildPhaseCargoCommand = "cargoWithProfile run --features user-doc -- user-doc ${docSrc}";
          });

        # Check formatting
        my-crate-fmt = craneLib.cargoFmt {
          inherit src;
        };

        # Audit dependencies
        my-crate-audit = craneLib.cargoAudit {
          inherit src advisory-db;
          cargoAuditExtraArgs = "--ignore RUSTSEC-2023-0071";
        };

        # Audit licenses
        my-crate-deny = craneLib.cargoDeny {
          inherit src;
        };

        # Run tests with cargo-nextest
        # Consider setting `doCheck = false` on `my-crate` if you do not want
        # the tests to run twice
        my-crate-nextest = craneLib.cargoNextest (commonArgs
          // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
      };

      packages =
        {
          default = my-crate;
        }
        // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
          my-crate-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs
            // {
              inherit cargoArtifacts;
            });
        };

      apps.default = flake-utils.lib.mkApp {
        drv = my-crate;
      };

      formatter = pkgs.alejandra;

      devShells.default = craneLib.devShell {
        # Inherit inputs from checks.
        checks = self.checks.${system};

        RUST_SRC_PATH = "${fenixToolChain.rust-src}/lib/rustlib/src/rust/library";

        # Additional dev-shell environment variables can be set directly
        # MY_CUSTOM_DEVELOPMENT_VAR = "something else";
        RENKU_CLI_RENKU_URL = "https://ci-renku-3689.dev.renku.ch";

        # Enable mold https://github.com/rui314/mold
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.clang}/bin/clang";
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C link-arg=-fuse-ld=${pkgs.mold}/bin/mold";

        # Need to set LD_LIBRARY_PATH for mold/clang, otherwise ssl is not linked
        # Not needed when using standard linker
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [pkgs.openssl];

        # Extra inputs can be added here; cargo and rustc are provided by default.
        packages = with pkgs; [
          cargo-edit
          cargo-expand
          tokio-console
          fenixToolChain.rust-analyzer
          fenixToolChain.rustfmt
        ];
      };
    })
    // {
      overlays.default = final: prev: {
        rnk = self.packages.${prev.system}.default;
      };
    };
}
