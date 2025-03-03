# Rich Text Parser

A WebAssembly-powered rich text and TTF font parser designed to work with Three.js for enhanced text rendering capabilities.

## Features

- TTF font file parsing using WebAssembly for optimal performance
- Rich text markup interpretation supporting:
  - Bold text `[b]...[/b]`
  - Italic text `[i]...[/i]`
  - Custom colors `[color=#RRGGBB]...[/color]`
- Integration with Three.js (coming soon)

## Development Status

Currently implements:
- TTF font parsing and glyph extraction
- Rich text markup lexer and parser
- WASM interface for font rendering

Planned features:
- Three.js RichTextGeometry implementation
- Enhanced text rendering capabilities
- Better alternative to existing TextGeometry

## Project Structure

- `src/` - Rust source code for font parsing and text interpretation
- `pkg/` - WebAssembly build output
- Rich text parsing in [`src/interpreter.rs`](src/interpreter.rs)
- Font rendering in [`src/lib.rs`](src/lib.rs)

## Building

Requires Rust and wasm-pack. Build the WebAssembly module with:

```sh
wasm-pack build
```