# cpt

A Windows command-line filter that copies standard input to both standard
output and the clipboard using Windows `clip.exe`.

## Usage

```cmd
uaxfmt.exe | cpt.exe
```

This is useful for GreenPad external filters: the formatted text is returned
to the editor and is also available for pasting into another application.

`cpt` passes the original bytes unchanged to standard output and to
`%SystemRoot%\System32\clip.exe`. It does not detect or convert character
encodings; clipboard interpretation follows `clip.exe`.
Empty input clears the clipboard.

No command-line arguments, help option, or version option are supported.
Errors are reported to standard error, with exit code 1. Success returns 0.
Processing stops on failure; output already written is not rolled back.

## Build

Install the Rust toolchain for Windows, then run:

```cmd
cargo build --release
```

The executable is `target\release\cpt.exe`. The release profile is optimized
for binary size.

## Tests

```cmd
cargo test
cargo clippy --all-targets -- -D warnings
```

The Japanese specification is in [SPEC.md](SPEC.md).
