# parse-obsidian

A Rust library for parsing and editing Obsidian vault files. Exposes Obsidian-specific markdown extensions programmatically for use outside the Obsidian app.

Obsidian files are markdown-based but include additional syntax and conventions that carry special meaning within the app and its plugin ecosystem. This library understands those constructs, allowing you to read, query, and modify them.

## Implemented

### Core Markdown (via comrak)
- Headings, paragraphs, lists
- Code blocks (inline and fenced)
- Links and images
- Emphasis, bold, strikethrough

## Not Implemented

### Obsidian Extensions
- Wikilinks: `[[Page Name]]` and `[[Page Name|Display Text]]`
- Tags: `#tag` and nested `#parent/child`
- Embeds: `![[file]]` and `![[file#heading]]`
- Callouts: `> [!note]` style blocks

### Plugin Conventions
- **Tasks plugin**: Checkbox items with emoji-based metadata
  - Due dates: `📅 2024-01-15`
  - Scheduled dates: `⏳ 2024-01-10`
  - Start dates: `🛫 2024-01-01`
  - Priority: `⏫` (high), `🔼` (medium), `🔽` (low), `⏬` (lowest)
  - Recurrence: `🔁 every week`
  - Done dates: `✅ 2024-01-15`

### Frontmatter
- YAML metadata block parsing
- Property access and modification
