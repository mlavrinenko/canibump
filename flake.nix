{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    qahq.url = "github:mlavrinenko/qahq";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    # The docs toolchain: Typst 0.14 and the tinymist (typlite) built on it,
    # the Typst that tola 0.7.1 embeds. A page that compiles under one must
    # compile under the others, so they move together. Same pin as mindtape.
    nixpkgs-typst.url = "github:NixOS/nixpkgs/9f11f828c213641c2369a9f1fa31fe31557e3156";
    # The Typst-native static site generator that builds www/.
    tola.url = "github:tola-rs/tola-ssg/v0.7.1";
  };

  outputs =
    {
      qahq,
      flake-utils,
      naersk,
      nixpkgs,
      nixpkgs-typst,
      tola,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };
        pkgs-typst = import nixpkgs-typst { inherit system; };

        naersk' = pkgs.callPackage naersk { };

        # The same major as rustc's: a mismatched llvm-profdata rejects the
        # profraw format.
        llvm = pkgs.llvmPackages_21.llvm;

        # The tools of `just check-code`, `cover`, `crap` and `deny`: the
        # Rust gates and what they shell out to.
        rustTools = [
          qahq.packages.${system}.cargo-crap
          qahq.packages.${system}.ejectest
          qahq.packages.${system}.jscpd
          qahq.packages.${system}.linecop
          # Memoized command runner. Every arm of `just check` runs as
          # `mmz just <subgate>`, so an arm whose declared inputs are
          # unchanged since it last passed is skipped. Rules and their input
          # scopes live in .mmz/config.yaml. `mmz --is-fresh --tag gate`
          # asserts a prior pass without running anything.
          qahq.packages.${system}.mmz
          qahq.packages.${system}.outdatty
        ] ++ (with pkgs; [
          rustc
          cargo
          cargo-llvm-cov
          cargo-machete
          cargo-deny
          # cargo-nextest is the test runner `just test` gates on; doctests
          # run in a separate `cargo test --doc` pass
          # (canibump:nextest-doctests), so `just test` spells out both.
          cargo-nextest
          # `just count-tests` parses nextest's JSON listing with jq.
          jq
          clippy
          rustfmt
          just
        ]);

        # The tools of `just check-docs` on top of the Rust ones, since the
        # captures run the binary.
        docsTools = [
          tola.packages.${system}.default
          # The docs site: typst queries each page for its metadata, typlite
          # (in tinymist) writes the Markdown copies and the README, and
          # typstyle parses the pages for the doc gates.
          pkgs-typst.typst
          pkgs-typst.tinymist
          pkgs-typst.typstyle
        ] ++ (with pkgs; [
          # pagefind indexes the built site for search. python3 runs the
          # mock upstream the docs captures ask (www/mock-upstream.py), and
          # util-linux's `script` gives each capture a terminal.
          pagefind
          python3
          util-linux
        ]);

        # Tools a contributor reaches for that no CI job runs.
        devTools = with pkgs; [
          cargo-mutants
          # The bump, CHANGELOG dating and commit behind `just release`.
          cargo-release
          # cargo-outdated is a dev aid, deliberately not wired into `just
          # check`: a new upstream release is not a reason for a gate to go
          # red.
          cargo-outdated
          # `wt` forks and lands worktrees: `just fork` and `just land` run
          # it, and .config/wt.toml holds the hooks it fires.
          worktrunk
          nixd
          rust-analyzer
        ];

        # canibump:llvm-cov-path-fallback
        llvmEnv = ''
          export LLVM_COV=${llvm}/bin/llvm-cov
          export LLVM_PROFDATA=${llvm}/bin/llvm-profdata
        '';
      in
      {
        # For `nix build` & `nix run`:
        packages.default = naersk'.buildPackage {
          src = ./.;
          nativeBuildInputs = [ pkgs.installShellFiles ];
          # Completions and the manifest schema come from the built binary, so
          # they cannot drift from the CLI or the manifest types.
          postInstall = ''
            installShellCompletion --cmd canibump \
              --bash <($out/bin/canibump completions bash) \
              --zsh <($out/bin/canibump completions zsh) \
              --fish <($out/bin/canibump completions fish)
            mkdir -p $out/share/canibump
            $out/bin/canibump schema > $out/share/canibump/canibump.schema.json
          '';
        };

        # CI enters the narrow shells, so a job realizes only the toolchain
        # it runs: `.#rust` for the code gates, `.#docs` for the site.
        devShells.rust = pkgs.mkShell {
          nativeBuildInputs = rustTools;
          shellHook = llvmEnv;
        };
        devShells.docs = pkgs.mkShell {
          nativeBuildInputs = rustTools ++ docsTools;
        };

        # For `nix develop`:
        devShells.default = pkgs.mkShell {
          # RUSTC_WRAPPER comes from the host session (e.g. a system-wide build
          # cache like kache); the dev shell inherits it and it never reaches the
          # `nix build` sandbox. No wrapper pinned here.
          nativeBuildInputs = rustTools ++ docsTools ++ devTools;
          shellHook = llvmEnv + ''
            # Wire the tracked pre-commit hook (.githooks/) on every shell
            # entry, so a fresh clone's first `nix develop` installs it and
            # a hooksPath edit is picked up live. Quiet: log noise would
            # print on every shell a contributor opens.
            just install-hooks > /dev/null 2>&1
          '';
        };
      }
    );
}
