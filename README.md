# genie-rs

Rust libraries for reading/writing various Age of Empires I/II files.

## Install

```toml
[dependencies]
genie = "^0.1.0"
```

## Example Programs

```bash
# Extract scenario files from a campaign to the working directory.
cargo run --example extractcpx ~/path/to/campaign.cpx

# Show the scenario files in a campaign file.
cargo run --example extractcpx ~/path/to/campaign.cpx -l

# Convert an HD Edition (+expansions) scenario to WololoKingdoms.
cargo run --example convertscx ~/path/to/input.aoe2scenario ~/path/to/output.scx wk

# Display contents of a language file.
cargo run --example displaylang ~/path/to/input/language.dll dll
cargo run --example displaylang ~/path/to/input/language.ini ini
cargo run --example displaylang ~/path/to/input/key-value-strings.txt key-value

# Convert HD Edition key-value.txt language files to language.ini files for Voobly or aoc-language-ini
cargo run --example wolololang ~/path/to/input/key-value-strings.txt ~/path/to/output/language.ini
```

## License

[GPL-3.0](./LICENSE.md)
