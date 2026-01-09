# parse-obsidian

A Rust library for parsing and editing Obsidian vault files. Exposes Obsidian-specific markdown extensions programmatically for use outside the Obsidian app.

Obsidian files are markdown-based but include additional syntax and conventions that carry special meaning within the app and its plugin ecosystem. This library understands those constructs, allowing you to read, query, and modify them.

## Implemented

### Core Markdown (via comrak)
- Headings, paragraphs, lists
- Code blocks (inline and fenced)
- Links and images
- Emphasis, bold, strikethrough

### Tasks Plugin
Parses checkbox items with emoji-based metadata. See [Tasks plugin documentation](https://publish.obsidian.md/tasks/).

- Due dates: `📅 2024-01-15`
- Scheduled dates: `⏳ 2024-01-10`
- Start dates: `🛫 2024-01-01`
- Priority: `⏫` (highest), `🔼` (high), `🔽` (low), `⏬` (lowest)
- Recurrence: `🔁 every week`
- Done dates: `✅ 2024-01-15`

### Tags
Parses inline tags including nested hierarchies.

- Simple tags: `#tag`
- Nested tags: `#parent/child/grandchild`
- Distinguishes from headings (`# Heading` is not a tag)

### Wikilinks
Parses internal links between notes.

- Simple links: `[[Page Name]]`
- Display text: `[[Page Name|click here]]`
- Heading anchors: `[[Page Name#Section]]`
- Combined: `[[Page Name#Section|display text]]`

## Not Implemented

### Obsidian Extensions
- Embeds: `![[file]]` and `![[file#heading]]`
- Callouts: `> [!note]` style blocks

### Frontmatter
- YAML metadata block parsing
- Property access and modification
