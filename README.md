# Checksum Generator

Checksum Generator is a command-line tool written in Rust that recursively generates checksum files for all files in a directory.

> **Note:** Designed for Linux/Unix systems (tested on Linux).

## Features

- **Recursive Processing:** Scans all files within a directory and its subdirectories.
- **Algorithm Support:** Supports MD5, SHA256, and BLAKE3 hash algorithms.
- **Security:** Generates read-only checksum files (mode 444) with timestamps.
- **Smart Filtering:** Automatically skips temporary (`.tmp-checksums_`) and existing (`checksums_`) files.
- **Performance:** Optimized with a configurable buffer size (8192 bytes default).

## Installation

```bash
git clone [https://github.com/tghezfs/cksum.git](https://github.com/tghezfs/cksum.git)
cd cksum
cargo build --release
```

**Binary location:** target/release/cksum

## Usage

```bash
./cksum --path <DIRECTORY> --algorithm <ALGORYTHM>
```

### Arguments

| Argument      | Description                                               |
| ------------- | --------------------------------------------------------- |
| `--path`      | Directory path to process (required)                      |
| `--algorithm` | Hash algorithm: md5, sha256, or blake3 (case-insensitive) |

### Examples

```bash
./cksum --path /home/user/documents --algorithm sha256
./cksum --path ./data --algorithm md5
./cksum --path ./projects --algorithm blake3
```

## Output

Creates a file named `checksums_YYYY-MM-DD-HHMMSS.txt` in the target directory:

**Format:**

```plaintext
<algorithm> <hash> <filename>
```

**Example:**

```plaintext
sha256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 example.txt
md5 5d41402abc4b2a76b9719d911017c592 document.pdf
```

## Testing

**Coverage:** ~75-80%

```bash
cargo test
```

### Covered

- All hash algorithms (empty and known strings)
- Content of varying sizes (2048 bytes) and small buffers (4 bytes)
- Empty directories and nested structures
- Non-existent paths and invalid algorithms
- Case-insensitive algorithm input
- Prefix skipping (.tmp-checksums\_, checksums\_)
- Read-only permissions (Unix)

### Not covered

- Very large files (>100MB)
- Special characters in filenames
- Unreadable files (permission denied)
- Symbolic links

## Requirements

- Rust 1.85.1+
- Linux/Unix only (uses Unix-specific APIs for permission handling)

## Dependencies

```toml
[dependencies]
blake3 = "1.8.4"
chrono = "0.4.44"
clap = { version = "4.6.0", features = ["derive"] }
md5 = "0.8.0"
sha2 = "0.11.0"
tempfile = "3.27.0"
walkdir = "2.5.0"

[dev-dependencies]
tempfile = "3.27.0"
walkdir = "2.5.0"
```

## Limitations

- **OS:** Linux/Unix only (permission handling uses Unix-specific APIs)
- **Buffer:** Fixed buffer size (8192 bytes, configurable in src/config.rs)

## Purpose

Educational project for learning Rust:

- File system operations
- Cryptographic hashing
- Error handling with Result and Box
- Unit and integration testing
- CLI structure with clap

## License

MIT
