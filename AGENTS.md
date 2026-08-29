# AGENTS.md

## Components

- vCard Rust library lives in `./components/lib/`;
- vCard CLI built with Rust lives in `./components/cli/`;
- Website hosted at option63.eu lives in `./components/web/`.

## Tool calls

- Run all Rust and NPM commands through `nix-shell`.

## Code

- New features must be made with corresponding tests.
- Bug fixes must be made with corresponding tests.
- Do not implement things that are not explicitly requested.
- Handle all errors properly, use early returns. Use the `thiserror` crate in Rust code.
