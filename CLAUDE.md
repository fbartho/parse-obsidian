# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust library for parsing Obsidian vault files. Parses Obsidian-specific markdown extensions (tasks, tags, wikilinks) that go beyond standard markdown.

## Commands

```bash
cargo build          # Build the library
cargo test           # Run all tests
cargo test tags      # Run tests for a specific module
cargo test test_name # Run a specific test by name
```

## Architecture

Each Obsidian construct has its own module under `src/`:

- **tasks.rs** - Parses Tasks plugin format (`- [ ] task 📅 2024-01-15`)
- **tags.rs** - Parses `#tag` and `#nested/tag` syntax
- **wikilinks.rs** - Parses `[[Page]]` and `[[Page#Heading|Display]]` links

Each module follows the same pattern:

- A struct for the parsed element (`Task`, `Tag`, `Wikilink`)
- `parse_*(input: &str) -> Vec<T>` - parse from text
- `find_*(path: impl AsRef<Path>) -> io::Result<Vec<T>>` - find in files/directories (uses `ignore` crate, respects .gitignore)

Re-exports in `lib.rs` expose the public API.

## Dependencies

- **comrak** - GitHub Flavored Markdown parsing (AST-based, supports round-trip editing)
- **chrono** - Date handling for Tasks plugin dates
- **ignore** - Directory walking that respects .gitignore
