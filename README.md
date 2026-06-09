# validate-encoding

MCP tool that validates file encoding and detects mojibake (garbled text) in Nordic/European text files.

## Tools

### `validate_file`

Validates a file's encoding and checks for common mojibake patterns.

| Argument | Type | Description |
|---|---|---|
| `file_path` | `string` | Path to the file to validate |
| `encoding` | `string?` | Expected encoding (e.g. `utf-8`, `iso-8859-1`, `windows-1252`) |

Returns a JSON report with detected encoding, confidence, valid encodings, git info, and any mojibake warnings.

## Installation

```bash
git clone https://github.com/Ochnee/validate-file-encoding
cd validate-file-encoding
./install.sh
```

This builds the shared library and installs it to `~/.config/rust-tools/`.

## Requirements

- [rust-tools-mcp](https://github.com/Ochnee/rust-tools-mcp) — the MCP host that loads this plugin
- Rust toolchain
