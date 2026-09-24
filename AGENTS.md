# AGENTS.md

## General agent behavior

- Respond with maximum brevity, eliminating conversational filler, and polite closings to conserve context tokens.
- Create temporary files in the current working directory. Do not use `/tmp`.
- Don't use em-dashes, use simple dashes, not "—", but "-".

## Components

- vCard Rust library lives in `./components/lib/`;
- vCard CLI built with Rust lives in `./components/cli/`;
- Website hosted at option63.eu lives in `./components/web/`.

## Tool calls

- Run all Rust and NPM commands through `nix-shell --run "command here"`.

## Code

- When creating functions with side-effects:
  - Delegate the logic to pure functions to keep things testable.
  - Use traits to make mocking easier for side-effects.
- Handle all errors properly, use early returns. Use the `thiserror` crate in Rust code. Runtime code paths must never panic.
- Do not use acronyms in code, specs, or docs. Prefer full descriptive names (e.g. `logRequestResponseBody`, not `logRRB`).
- Secrets, API keys and other sensitive data must live in files, never environment variables or CLI flags.
- Testing rules (what/when to test, thread safety, test commands, coverage expectations) live in `specs/testing.md` - follow it.

## Specifications

Specs are the project's source of truth for what we build and how. Before implementing a feature or significant change, write a spec and get it reviewed, then implement to its success criteria. Specs live under `./specs/`. They are living specs, permanent sources of truththat describe how the system is built and behaves. They are edited as the system evolves.

## RFC

- RFC files can be listed with the command `rfc-list`.
- An RFC file can be downloaded with `rfc-download <number>`, eg `rfc-download 6352`.
