//! Parser for YAML frontmatter in Obsidian markdown files.
//!
//! Obsidian supports YAML metadata at the top of files, delimited by `---`.

use ignore::WalkBuilder;
use serde_yml::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// Parsed frontmatter as a map of property names to values.
pub type Frontmatter = HashMap<String, Value>;

/// Extracts and parses frontmatter from the given text.
///
/// Returns `None` if there is no frontmatter block.
/// Returns `Some(HashMap)` with the parsed properties, or an empty map if
/// the frontmatter block exists but is empty.
pub fn parse_frontmatter(input: &str) -> Option<Frontmatter> {
    let trimmed = input.trim_start();

    // Must start with ---
    if !trimmed.starts_with("---") {
        return None;
    }

    // Find the end of the first line (the opening ---)
    let after_open = &trimmed[3..];
    let content_start = after_open.find('\n').map(|i| i + 1)?;

    // Find the closing ---
    let content = &after_open[content_start..];
    let end_pos = find_closing_delimiter(content)?;

    let yaml_content = &content[..end_pos];

    // Empty frontmatter is valid
    if yaml_content.trim().is_empty() {
        return Some(HashMap::new());
    }

    // Parse the YAML
    serde_yml::from_str(yaml_content).ok()
}

fn find_closing_delimiter(content: &str) -> Option<usize> {
    for (i, line) in content.lines().enumerate() {
        if line.trim() == "---" {
            // Calculate byte position
            let pos: usize = content
                .lines()
                .take(i)
                .map(|l| l.len() + 1) // +1 for newline
                .sum();
            return Some(pos);
        }
    }
    None
}

/// Finds and parses frontmatter from markdown files at the given path.
///
/// If `path` is a file, parses frontmatter from that file.
/// If `path` is a directory, recursively walks it and parses all `.md` files.
/// Respects `.gitignore` and other ignore files.
///
/// Returns a vec of (path, frontmatter) pairs for files that have frontmatter.
pub fn find_frontmatter<P: AsRef<Path>>(path: P) -> io::Result<Vec<(std::path::PathBuf, Frontmatter)>> {
    let path = path.as_ref();
    let mut results = Vec::new();

    if path.is_file() {
        if path.extension().is_some_and(|ext| ext == "md") {
            let content = fs::read_to_string(path)?;
            if let Some(fm) = parse_frontmatter(&content) {
                results.push((path.to_path_buf(), fm));
            }
        }
    } else if path.is_dir() {
        for entry in WalkBuilder::new(path).build() {
            let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let entry_path = entry.path();
            if entry_path.is_file() && entry_path.extension().is_some_and(|ext| ext == "md") {
                let content = fs::read_to_string(entry_path)?;
                if let Some(fm) = parse_frontmatter(&content) {
                    results.push((entry_path.to_path_buf(), fm));
                }
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_frontmatter() {
        let input = r#"---
title: My Note
---

Content here"#;
        let fm = parse_frontmatter(input).unwrap();
        assert_eq!(fm.get("title").unwrap(), "My Note");
    }

    #[test]
    fn test_multiple_properties() {
        let input = r#"---
title: My Note
author: John
year: 2024
---

Content"#;
        let fm = parse_frontmatter(input).unwrap();
        assert_eq!(fm.get("title").unwrap(), "My Note");
        assert_eq!(fm.get("author").unwrap(), "John");
        assert_eq!(fm.get("year").unwrap(), 2024);
    }

    #[test]
    fn test_tags_array() {
        let input = r#"---
tags:
  - rust
  - programming
  - obsidian
---
"#;
        let fm = parse_frontmatter(input).unwrap();
        let tags = fm.get("tags").unwrap().as_sequence().unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0], "rust");
    }

    #[test]
    fn test_inline_array() {
        let input = r#"---
tags: [one, two, three]
---
"#;
        let fm = parse_frontmatter(input).unwrap();
        let tags = fm.get("tags").unwrap().as_sequence().unwrap();
        assert_eq!(tags.len(), 3);
    }

    #[test]
    fn test_aliases() {
        let input = r#"---
aliases:
  - alias1
  - alias2
---
"#;
        let fm = parse_frontmatter(input).unwrap();
        let aliases = fm.get("aliases").unwrap().as_sequence().unwrap();
        assert_eq!(aliases.len(), 2);
    }

    #[test]
    fn test_nested_object() {
        let input = r#"---
metadata:
  created: 2024-01-01
  modified: 2024-01-15
---
"#;
        let fm = parse_frontmatter(input).unwrap();
        let metadata = fm.get("metadata").unwrap().as_mapping().unwrap();
        assert_eq!(metadata.get("created").unwrap(), "2024-01-01");
    }

    #[test]
    fn test_boolean_values() {
        let input = r#"---
draft: true
publish: false
---
"#;
        let fm = parse_frontmatter(input).unwrap();
        assert_eq!(fm.get("draft").unwrap().as_bool(), Some(true));
        assert_eq!(fm.get("publish").unwrap().as_bool(), Some(false));
    }

    #[test]
    fn test_no_frontmatter() {
        let input = "Just regular content without frontmatter";
        assert!(parse_frontmatter(input).is_none());
    }

    #[test]
    fn test_empty_frontmatter() {
        let input = r#"---
---

Content"#;
        let fm = parse_frontmatter(input).unwrap();
        assert!(fm.is_empty());
    }

    #[test]
    fn test_frontmatter_with_leading_whitespace() {
        let input = r#"
---
title: Note
---
"#;
        let fm = parse_frontmatter(input).unwrap();
        assert_eq!(fm.get("title").unwrap(), "Note");
    }

    #[test]
    fn test_unclosed_frontmatter() {
        let input = r#"---
title: Unclosed
no closing delimiter"#;
        assert!(parse_frontmatter(input).is_none());
    }

    #[test]
    fn test_dashes_in_content() {
        let input = r#"---
title: Note
---

Some content with --- dashes in it"#;
        let fm = parse_frontmatter(input).unwrap();
        assert_eq!(fm.get("title").unwrap(), "Note");
    }

    #[test]
    fn test_multiline_string() {
        let input = r#"---
description: |
  This is a
  multiline description
---
"#;
        let fm = parse_frontmatter(input).unwrap();
        let desc = fm.get("description").unwrap().as_str().unwrap();
        assert!(desc.contains("multiline"));
    }

    #[test]
    fn test_null_value() {
        let input = r#"---
empty_field:
---
"#;
        let fm = parse_frontmatter(input).unwrap();
        assert!(fm.get("empty_field").unwrap().is_null());
    }

    #[test]
    fn test_quoted_string() {
        let input = r#"---
title: "A: Colon in Title"
---
"#;
        let fm = parse_frontmatter(input).unwrap();
        assert_eq!(fm.get("title").unwrap(), "A: Colon in Title");
    }

    #[test]
    fn test_date_as_string() {
        let input = r#"---
date: 2024-01-15
---
"#;
        let fm = parse_frontmatter(input).unwrap();
        // YAML may parse this as a string or date depending on implementation
        let date = fm.get("date").unwrap();
        assert!(date.is_string() || date.as_str().is_some() || format!("{:?}", date).contains("2024"));
    }
}
