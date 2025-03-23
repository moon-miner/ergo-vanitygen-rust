# 🚀 Ergo Vanitygen

A high-performance tool to create custom Ergo blockchain addresses with patterns of your choice. Built in Rust for speed and inspired by [ergo-vanitygen](https://github.com/jellymlg/ergo-vanitygen).

## 🔍 What Is Ergo Vanitygen?

Want your Ergo address to contain your name, a word, or your favorite number? This tool makes that possible by searching through possible addresses until it finds one that matches your desired pattern.

## 🔥 Key Features

✅ **User-friendly interface** – Choose between a GUI or command line mode  
✅ **Pattern matching flexibility** – Find patterns at the beginning, end, or anywhere in the address  
✅ **Fast processing** – Multi-threaded design utilizes all available CPU cores  
✅ **Customizable seed phrases** – Supports 12, 15, or 24-word seed phrases  
✅ **Real-time feedback** – Watch progress and matches in real-time  
✅ **Cold storage support** – Generate secure paper wallets  
✅ **Cross-platform compatibility** – Runs on Windows, Linux, and macOS  

## 🏃 Quick Start

1. **Download** the latest release from the [releases page](https://github.com/arkadianet/ergo-vanitygen/releases)
2. **Run** the application (double-click the file or run via terminal)
3. **Enter** your desired pattern(s)
4. **Click** Start Search — and watch the magic happen!

## 📚 Understanding Ergo Addresses

Ergo addresses have a specific format:

* Mainnet addresses start with `9`
* The second character will be one of: `e`, `f`, `g`, `h`, `i`
* Example: `9eXo2H3mZkKgqB...`

> ⚠️ If you want to search for a pattern at the beginning, it must follow the `9` and start with a valid second character.

## 🛠️ Installation Options

### ✅ Easy Way: Pre-built Binaries

| Platform | Download |
|----------|----------|
| Windows | Download the `.exe` file from the [releases page](https://github.com/arkadianet/ergo-vanitygen/releases) |
| Linux | Download the standard executable or `.AppImage` (no installation required) |
| macOS | Coming soon! |

### 👨‍💻 For Developers: Build From Source

Clone the repository and build using cargo:

```bash
git clone https://github.com/arkadianet/ergo-vanitygen-rust
cd ergo-vanitygen-rust
cargo build --release
```

Optimize for your hardware (use native CPU instructions for best performance):

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## 💡 Usage Guide

### GUI Mode

* Launch the application (GUI opens by default)
* Enter pattern(s), adjust settings, and hit Start
* Copy generated addresses and seed phrases directly from the interface

### Command Line Mode

Customize your search directly from the terminal:

```bash
ergo-vanitygen -p your_pattern
```

#### Common Options:

| Option | Description |
|--------|-------------|
| `-p, --pattern` | Pattern(s) to search for (comma-separated) |
| `-s, --start` | Match pattern at the start of the address |
| `-e, --end` | Match pattern at the end of the address |
| `-m, --matchCase` | Case-sensitive search |
| `-i, --index <number>` | Addresses to check per seed (default: 1) |
| `-n, --num <number>` | Number of matches to find (default: 1) |
| `--w12` | Use 12-word seed for faster generation |
| `--estimate` | Estimate time/difficulty before starting |
| `--no-gui` | Force command-line mode |

## 🧪 Pattern Matching Examples

Find an address with "cafe" at the end:

```bash
ergo-vanitygen -e -p cafe
```

Find an address starting with "ergo":

```bash
ergo-vanitygen -s -p ergo
```

Find multiple patterns in one search:

```bash
ergo-vanitygen -p coffee,tea,milk -n 3
```

## 📈 Performance

The tool scales based on your hardware:

* Mid-range CPU → ~8,000 addresses/second
* High-end CPU → ~15,000+ addresses/second

You can increase throughput using the `-i` option to test multiple addresses per seed.

## 🔒 Security

* All seeds are generated locally — nothing is transmitted online
* Industry-standard derivation (m/44'/429'/0'/0/X)
* Option to create paper wallets for cold storage

## 🎯 Difficulty Estimation

Estimate the time and attempts needed to find a match:

```bash
ergo-vanitygen -s -p ergo,humble --estimate
```

Example Output:

```
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

## 🛡️ Need Help?

* Open an issue on GitHub
* Check out the FAQ

## 📄 License

This project is licensed under the MIT License – see the LICENSE file for details.

## 🙌 Acknowledgments

Special thanks to:

* [Ergo Platform](https://ergoplatform.org/)
* [sigma-rust](https://github.com/ergoplatform/sigma-rust)
* Original [ergo-vanitygen](https://github.com/jellymlg/ergo-vanitygen) by jellymlg

## ⚠️ Disclaimer

This tool is for educational and entertainment purposes.  
Always verify generated addresses before using them.  
The authors are not responsible for any loss of funds. 
