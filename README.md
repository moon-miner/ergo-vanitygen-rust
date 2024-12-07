# Ergo Vanitygen

A high-performance vanity address generator for Ergo blockchain, written in Rust. This is a reimplementation and optimization of the original [ergo-vanitygen](https://github.com/jellymlg/ergo-vanitygen) by jellymlg.

## Features

- Generate Ergo addresses matching specific patterns
- Check multiple addresses from the same seed (new!)
- Support for both 12 and 24-word seed phrases
- Multi-threaded processing for optimal performance
- Case-sensitive and case-insensitive matching
- Match patterns at start, end, or anywhere in addresses
- Real-time progress monitoring with seed and address rates
- Real-time display of matches as they are found
- Performance statistics

## Address Format

Ergo P2PK addresses follow a specific format:
- Always start with '9' (mainnet prefix)
- Second character is always one of: e, f, g, h, i
- Example: 9eXo2H3mZkKgqB...

## Installation

### Pre-built Binary
Download the latest Windows release from the [releases page](https://github.com/arkadianet/ergo-vanitygen/releases).
Note: Pre-built binaries are compiled without CPU-specific optimizations for maximum compatibility. For best performance, consider building from source with CPU-native optimizations enabled.

### Building from source

Prerequisites:
- Rust toolchain (1.70.0 or later)
- Cargo package manager

```bash
git clone https://github.com/arkadianet/ergo-vanitygen-rust
cd ergo-vanitygen-rust

# For standard build
cargo build --release

# For optimized build with CPU-specific instructions (recommended, ~8.5% faster)
# Linux/macOS:
RUSTFLAGS="-C target-cpu=native" cargo build --release
# Windows PowerShell:
$env:RUSTFLAGS="-C target-cpu=native"; cargo build --release
```

The compiled binary will be available at `target/release/ergo-vanitygen-rust.exe`.

Note: The CPU-native build will enable optimizations specific to your processor, potentially improving performance by ~8.5% (primarily in address generation). However, the resulting binary may not be portable to other computers with different CPU architectures.

## Usage

```bash
# Basic usage (find pattern anywhere in address)
ergo-vanitygen -p <pattern>

# Options
-p, --pattern <pattern>    Patterns to look for (comma-separated)
-s, --start               Look for pattern at start
-e, --end                 Look for pattern at end
-m, --matchCase           Match pattern with case sensitivity
-i, --index <number>      Number of addresses to check per seed (default: 1)
    --w12                 Generate 12-word seed phrases (default: 24)
-n, --num <number>        Number of matches to find (default: 1)
-b, --balanced           Try to find matches for all patterns evenly
    --estimate           Estimate time to find matches before starting
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

3. Anywhere matching (default, no -s or -e):
   - No restrictions on pattern
   - Example: `-p lucky` will find addresses containing "lucky" anywhere

### Examples

1. Find an address starting with "ergo", checking first 10 addresses from each seed:
```bash
ergo-vanitygen -s -p ergo -i 10
```

2. Find an address ending with "cafe" in first 5 positions:
```bash
ergo-vanitygen -e -p cafe -i 5
```

3. Find an address containing "lucky" using 12-word seed, checking 20 addresses per seed:
```bash
ergo-vanitygen -p lucky --w12 -i 20
```

4. Find multiple patterns across first 10 addresses of each seed:
```bash
ergo-vanitygen -p ergo,sigma -i 10 -n 5 -b
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
ergo-vanitygen -e -p cafe,shop,mart -n 3 -b
```

## Performance

The generator is optimized for modern multi-core processors:
- Utilizes all available CPU cores
- Efficient batch processing
- Reuses master key for multiple addresses from same seed
- Real-time progress monitoring showing both seed and address rates
- Multiple result collection

Tested performance (single address per seed):
- Mid-range CPU (6-8 cores): ~8,000 addresses/second
- High-end CPU (12+ cores): ~15,000 addresses/second

Tested performance (with -i 10) Using CPU-native optimizations (RUSTFLAGS="-C target-cpu=native"):
- AMD Ryzen 7 7800X3D (16 threads): ~4,300 seeds/second (~43,000 addresses/second)
- AMD Ryzen 9 5950x (32 threads): ~7,100 sees/second (~71,000 adresses/second)

Note: Actual performance will vary based on your system specifications and build options.
Performance may be lower when collecting multiple results as the program
continues searching until all requested matches are found.

## Security Notes

- All seed phrases are generated securely using system entropy
- Implements BIP39 for mnemonic generation
- Follows EIP-3 for Ergo address derivation (m/44'/429'/0'/0/X)
- No seed phrases are stored or transmitted
- Each seed can generate multiple addresses using standard derivation paths

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

## Difficulty Estimation
```bash
# Estimate time to find matches
ergo-vanitygen -s -p ergo,humble --estimate

# Example output:
Difficulty Estimation
====================
Pattern: "ergo"
Estimated attempts needed: 3,125
Estimated time to find:
  At 10,000 addr/s: 0.3 seconds
  At 20,000 addr/s: 0.2 seconds

Pattern: "humble"
Estimated attempts needed: 15,625
Estimated time to find:
  At 10,000 addr/s: 1.6 seconds
  At 20,000 addr/s: 0.8 seconds
``` 
