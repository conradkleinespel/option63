# option63

**option63** is a toolkit for private contact, calendar, and email sharing.

## Services

**option63** is offered with **custom development, hosting, and managed services**. If you need a tailored solution, reach out: **contact@option63.eu**

## Components

**option63** consists of 2 main components:
- a Rust library for vCard parsing & display, within `vcard-lib/`;
- a command line interface for common vCard operations, within `vcard-bin/`.

> **Note**: This project is in its **early stages** and is **experimental**. Features as well as API interfaces will stabilize with time and feedback.

## Rust library usage

You'll need to have **Rust** installed on your system.

```bash
git clone https://github.com/conradkleinespel/option63.git && cd option63
cargo doc --package vcard-lib
```

Open the HTML file created by `cargo doc` to browse documentation.

## CLI usage

You'll need to have **Rust** installed on your system.

```bash
git clone https://github.com/conradkleinespel/option63.git && cd option63

# show CLI commands and options
cargo run -- --help

# validate and display a vCard as parsed by option63
cargo run -- show contacts.vcf
# keep only an allow-list of properties from a vCard
# useful to create ad-hoc vCards to preserve the privacy of your contacts in some environments
cargo run -- show contacts.vcf --props FN,N,TEL,EMAIL
```

## License

This project - excluding its test suite - is licensed under the **GNU Affero General Public License v3.0 (AGPLv3)**. See the [LICENSE](LICENSE) file for details. The test suite is excluded from this license, remains private, and is not subject to AGPLv3.

Custom licensing is available for this project. If you're looking for a different license that would allow you to use this package under different terms, please [reach out via email](mailto:conradk@conradk.com).
