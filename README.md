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

### Pre-built Binary
Download the latest Windows release from the [releases page](https://github.com/arkadianet/ergo-vanitygen/releases).

### Building from source

Prerequisites:
- Rust toolchain (1.70.0 or later)
- Cargo package manager

```bash
git clone https://github.com/arkadianet/ergo-vanitygen-rust
cd ergo-vanitygen-rust
cargo build --release
```

The compiled binary will be available at `target/release/ergo-vanitygen-rust.exe`.

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
-n, --num <number>        Number of matching addresses to find (default: 1)
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

1. Find an address starting with "ergo":
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

4. Find an address ending with "ERGO" (case-sensitive):
```bash
ergo-vanitygen -e -p ERGO -m
```

5. Find five addresses starting with "ergo":
```bash
ergo-vanitygen -s -p ergo -n 5
```

6. Find three addresses ending with "cafe" (case-insensitive):
```bash
ergo-vanitygen -e -p cafe -n 3
```

7. Find addresses starting with either "humble" or "index":
```bash
ergo-vanitygen -s -p humble,index -n 2
```

8. Find three addresses ending with different words:
```bash
ergo-vanitygen -e -p cafe,shop,mart -n 3
```

## Performance

The generator is optimized for modern multi-core processors:
- Utilizes all available CPU cores
- Efficient batch processing
- Minimal memory overhead
- Real-time progress monitoring
- Multiple result collection

Tested performance:
- Mid-range CPU (6-8 cores): ~8,000 addresses/second
- High-end CPU (12+ cores): ~15,000 addresses/second

Note: Actual performance will vary based on your system specifications.
Performance may be lower when collecting multiple results as the program
continues searching until all requested matches are found.

## Security Notes

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
- Original [ergo-vanitygen](https://github.com/jellymlg/ergo-vanitygen) by jellymlg

## Disclaimer

This tool is for educational and entertainment purposes. Always verify generated addresses before use. The authors are not responsible for any loss of funds. 