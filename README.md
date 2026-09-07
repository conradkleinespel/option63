# option63

**option63** is a toolkit for manipulating vCard data. Type-safe Rust library and CLI. More stuff coming.

## Rust library usage

You'll need to have **Rust** installed on your system.

```bash
git clone https://github.com/conradkleinespel/option63.git && cd option63
cargo doc --package o63
```

Open the HTML file created by `cargo doc` to browse documentation.

## CLI usage

You'll need to have **Rust** installed on your system.

```bash
git clone https://github.com/conradkleinespel/option63.git && cd option63

# show CLI commands and options
cargo run -- --help

# validate and display a vCard as parsed by option63
cargo run -- vcard show contacts.vcf
# keep only an allow-list of properties from a vCard
# useful to create ad-hoc vCards to preserve the privacy of your contacts in some environments
cargo run -- vcard show contacts.vcf --props FN,N,TEL,EMAIL
# drop a property (optionally only where the value matches a regex)
cargo run -- vcard drop contacts.vcf TEL
# drop TEL properties whose value matches an area code, e.g. +1 555 area code
cargo run -- vcard drop contacts.vcf TEL --regex '^\+1-555'

# print base64-encoded upstream credentials as O63_CARDDAV_PROXY_* variables
cargo run -- carddav creds
# run a CardDAV reverse proxy forwarding traffic to an upstream server
cargo run -- carddav proxy --upstream https://dav.example.com
```

## License and services

This project - excluding its test suite - is licensed under the **GNU Affero General Public License v3.0 (AGPLv3)**. See the [LICENSE](LICENSE) file for details. The test suite is excluded from this license, remains private, and is not subject to AGPLv3.

Custom licensing is available. Looking for a different license? Need custom development, custom licensing, hosting, or managed services? [Reach out via email](mailto:contact@option63.eu).
