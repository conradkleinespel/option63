---
name: weekly-update
description: Drafts a short post summarizing user-visible changes since the last update. Use when the user asks for a weekly update, changelog, post, or "what changed since" for the option63-eu project.
---

# Weekly Update

Drafts a concise user-facing changelog post for the option63-eu project vCard library, CLI and website since the last published update.

## Baseline

- The update range is `<since>..HEAD`. HEAD is the current checked-out commit when the skill runs.
- Always ask the user for the previous baseline commit. This is the `<since>` end of the range: the last commit already covered by a prior update, so that this update only covers commits since then. Do not infer it from tags or any stored marker. There is no automated baseline tracking.

## Steps

1. Ask the user for the previous baseline commit and use it as the `since` end of the range.
2. Run `git log --oneline <since>..HEAD` to list commits in the range.
3. Read relevant diffs with `git show --stat <hash>` for any user-visible feature or fix so descriptions are accurate.
4. Filter to user-visible changes only. Exclude `chore`, `ci`, `refactor`, `style`, `devenv`, `misc`, doc-only and dependency updates unless they have clear user impact. Group by area, for example CLI, vCard parsing and website.
5. Verify all command invocations against the actual CLI source in `components/cli/src/main.rs` (clap `Command::new` definitions) rather than inferring them from commit messages. Use the real format, e.g. grouped subcommands like "o63 carddav proxy", not hyphenated guesses.

## Output rules

- A short post that opens with an emoji plus a light variant of   "Shipped a bunch of new things for Option63 over the last few days", reworded to avoid repeating the same phrasing week to week.
- Directly below the opening line, add a second context line that always starts with "For context" unmodified, e.g. "For context, Option63 is tooling that aims to give companies and individuals more robust, privacy-aware options for contact, calendar and email management" so new readers have context before the changelog.
- Group changes under short headers, one per area, each starting with a relevant emoji, e.g.:
  - **⚙️ CLI**
  - **📇 vCard parsing**
  - **🌐 Website**
- Use plain, concrete language. Describe what the change does, not how.
- For commands and flags, use the real invocation (starting with o63) enclosed in double quotes in plain text, with no markdown code formatting and no backticks around them. Put the flag itself in double quotes too. For example write "o63 vcard drop --strict" and "--strict", not `o63 vcard drop --strict`. Say what the flag does, such as "allows enforcing strict vCard spec conformance".
- Group closely related fixes into a single bullet rather than listing each separately, e.g. "More robust parsing of MEDIATYPE and GENDER properties."
- When a commit (or set of commits) introduces a feature with several subcommands or flags, highlight only the most notable command; don't enumerate supporting sibling subcommands (e.g. skip "o63 carddav creds" when covering the proxy).
- When a feature is early-stage or incomplete, say so and hint at what's next, e.g. "Still early days. More coming on that front." Ask the user which features (if any) to describe as early-stage instead of deciding yourself.
- Keep area sections short and low-detail; a couple of bullets per section is enough. Omit port numbers, default addresses, log levels, and other implementation detail.
- Keep the website note very brief, for example "Now works much better with JavaScript disabled." Only describe the user-visible effect, not the how.
- Do not include a link to option63.eu in the visible text. State that the relevant links will be included in a comment on the post.
- Do not include internal build or dev-only notes.
- In addition to the post, output the content for a comment on the post that carries the links that were excluded from the visible text, including a link to the project on GitHub (`https://github.com/conradkleinespel/option63`) and a link to the website (`https://option63.eu`). Ask the user whether any other links are relevant to include. Keep the comment short.
- Output only the post itself followed by the comment content to the console for copy-paste — no preamble sentence, header, or closing commentary. Do not write a file. Separate the two clearly, e.g. with a divider, and label the second block as the comment.
