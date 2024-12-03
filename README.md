# Ergo Vanitygen

A high-performance vanity address generator for Ergo blockchain, written in Rust. This is a reimplementation and optimization of the original [ergo-vanitygen](https://github.com/jellymlg/ergo-vanitygen) by jellymlg.

## Features

- Generate Ergo addresses matching specific patterns
- Support for both 12 and 24-word seed phrases
- Multi-threaded processing for optimal performance
- Case-sensitive and case-insensitive matching
- Match patterns at start or end of addresses
- Real-time progress monitoring
- Performance statistics

## Address Format

Ergo P2PK addresses follow a specific format:
- Always start with '9' (mainnet prefix)
- Second character is always one of: e, f, g, h, i
- Example: 9eXo2H3mZkKgqB...

## Installation

### Prerequisites

- Rust toolchain (1.70.0 or later)
- Cargo package manager

### Building from source

```bash
git clone https://github.com/yourusername/ergo-vanitygen-rust
cd ergo-vanitygen-rust
cargo build --release
```

The compiled binary will be available at `target/release/ergo-vanitygen`.

## Usage

```bash
# Basic usage (find pattern anywhere in address)
ergo-vanitygen -p <pattern>

# Options
-p, --pattern <pattern>    Pattern to look for in addresses
-s, --start               Look for pattern at start (must start with e,f,g,h,i)
-e, --end                 Look for pattern at end of addresses
-m, --matchCase           Match pattern with case sensitivity
    --w12                 Generate 12-word seed phrases (default is 24)
```

### Pattern Matching Rules

1. Start matching (-s/--start):
   - Pattern MUST start with one of: e, f, g, h, i
   - Will match after the '9' prefix
   - Example: `-s -p ergo` will find addresses like "9ergo..."
   - Invalid: `-s -p lucky` (must start with e,f,g,h,i)

2. End matching (-e/--end):
   - No restrictions on pattern
   - Example: `-e -p cafe` will find addresses ending with "cafe"

3. Anywhere matching (default):
   - No restrictions on pattern
   - Example: `-p lucky` will find addresses containing "lucky"

### Examples

1. Find an address starting with "ergo" (valid):
```bash
ergo-vanitygen -s -p ergo
```

2. Find an address ending with "cafe" (case-insensitive):
```bash
ergo-vanitygen -e -p cafe
```

3. Find an address containing "lucky" using 12-word seed:
```bash
ergo-vanitygen -p lucky --w12
```

4. Invalid example (will show error):
```bash
ergo-vanitygen -s -p lucky  # Error: must start with e,f,g,h,i
```

## Performance

The generator is optimized for modern multi-core processors:
- Utilizes all available CPU cores
- Efficient batch processing
- Minimal memory overhead
- Real-time performance monitoring

Typical performance varies by system:
- Modern desktop CPU: 5,000-15,000 addresses/second
- High-end CPU: 10,000-20,000 addresses/second

## Security

- All seed phrases are generated securely using system entropy
- Implements BIP39 for mnemonic generation
- Follows EIP-3 for Ergo address derivation (m/44'/429'/0'/0/0)
- No seed phrases are stored or transmitted

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Ergo Platform](https://ergoplatform.org/)
- [sigma-rust](https://github.com/ergoplatform/sigma-rust)
- [ergo-lib](https://github.com/ergoplatform/sigma-rust/tree/develop/ergo-lib)

## Disclaimer

This tool is for educational and entertainment purposes. Always verify generated addresses before use. The authors are not responsible for any loss of funds. 