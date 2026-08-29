# AGENTS.md

## Components

- vCard Rust library lives in `./components/lib/`;
- vCard CLI built with Rust lives in `./components/cli/`;
- Website hosted at option63.eu lives in `./components/web/`.

## Tool calls

- Run all Rust and NPM commands through `nix-shell`.

## Code

- New features must be made with corresponding tests. No need to test logging.
- Bug fixes must be made with corresponding tests.
- Do not implement things that are not explicitly requested.
- Code must be formatted with `cargo fmt`.
- Code must pass `cargo clippy` lints.
- Handle all errors properly, use early returns. Use the `thiserror` crate in Rust code. Runtime code paths must never panic.

## Specifications / RFC

- RFC files can be listed with the command `rfc-list`.
- An RFC file can be downloaded with `rfc-download <number>`, eg `rfc-download 6352`.