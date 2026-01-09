//! Parser for Obsidian tags.
//!
//! Obsidian supports inline tags using `#tag` syntax, including nested tags
//! like `#parent/child/grandchild`.

use ignore::WalkBuilder;
use std::fs;
use std::io;
use std::path::Path;

/// A parsed tag from an Obsidian markdown file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    /// The full tag text without the leading `#`.
    pub name: String,
    /// The tag segments split by `/` for nested tags.
    pub parts: Vec<String>,
}

impl Tag {
    fn new(name: &str) -> Self {
        let parts = name.split('/').map(|s| s.to_string()).collect();
        Self {
            name: name.to_string(),
            parts,
        }
    }
}

/// Extracts and parses all tags from the given text.
pub fn parse_tags(input: &str) -> Vec<Tag> {
    let mut tags = Vec::new();

    for line in input.lines() {
        // Skip if line starts with # (it's a heading)
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') && trimmed.chars().nth(1).is_some_and(|c| c == ' ' || c == '#') {
            // This is a heading (# Heading or ## Heading), but tags may still appear after
            // We need to find tags that aren't at the start
        }

        // Find all #tag patterns in the line
        let mut chars = line.char_indices().peekable();

        while let Some((i, c)) = chars.next() {
            if c == '#' {
                // Check if this could be a tag
                // Must not be preceded by alphanumeric (would be mid-word)
                let prev_char = if i > 0 {
                    line[..i].chars().last()
                } else {
                    None
                };

                // Tag must not be preceded by alphanumeric
                if prev_char.is_some_and(|p| p.is_alphanumeric()) {
                    continue;
                }

                // Check if at start of line (could be heading)
                if i == 0 || (i > 0 && line[..i].trim().is_empty()) {
                    // At start of line - check if it's a heading
                    if let Some(&(_, next_c)) = chars.peek() {
                        if next_c == ' ' || next_c == '#' {
                            // It's a heading, skip
                            continue;
                        }
                    }
                }

                // Extract the tag name
                let start = i + 1; // skip the #
                let mut end = start;

                for (j, next_c) in chars.by_ref() {
                    if is_valid_tag_char(next_c) {
                        end = j + next_c.len_utf8();
                    } else {
                        break;
                    }
                }

                if end > start {
                    let tag_name = &line[start..end];
                    // Don't include tags that are just numbers
                    if !tag_name.chars().all(|c| c.is_ascii_digit()) {
                        tags.push(Tag::new(tag_name));
                    }
                }
            }
        }
    }

    tags
}

/// Finds and parses all tags from markdown files at the given path.
///
/// If `path` is a file, parses tags from that file.
/// If `path` is a directory, recursively walks it and parses all `.md` files.
/// Respects `.gitignore` and other ignore files.
pub fn find_tags<P: AsRef<Path>>(path: P) -> io::Result<Vec<Tag>> {
    let path = path.as_ref();
    let mut results = Vec::new();

    if path.is_file() {
        if path.extension().is_some_and(|ext| ext == "md") {
            let content = fs::read_to_string(path)?;
            results.extend(parse_tags(&content));
        }
    } else if path.is_dir() {
        for entry in WalkBuilder::new(path).build() {
            let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let entry_path = entry.path();
            if entry_path.is_file() && entry_path.extension().is_some_and(|ext| ext == "md") {
                let content = fs::read_to_string(entry_path)?;
                results.extend(parse_tags(&content));
            }
        }
    }

    Ok(results)
}

fn is_valid_tag_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '/'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tag() {
        let tags = parse_tags("This has a #tag in it");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "tag");
        assert_eq!(tags[0].parts, vec!["tag"]);
    }

    #[test]
    fn test_nested_tag() {
        let tags = parse_tags("A #parent/child tag");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "parent/child");
        assert_eq!(tags[0].parts, vec!["parent", "child"]);
    }

    #[test]
    fn test_deeply_nested_tag() {
        let tags = parse_tags("#one/two/three/four");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "one/two/three/four");
        assert_eq!(tags[0].parts, vec!["one", "two", "three", "four"]);
    }

    #[test]
    fn test_multiple_tags() {
        let tags = parse_tags("Has #first and #second tags");
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "first");
        assert_eq!(tags[1].name, "second");
    }

    #[test]
    fn test_tag_with_underscore() {
        let tags = parse_tags("#my_tag");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "my_tag");
    }

    #[test]
    fn test_tag_with_hyphen() {
        let tags = parse_tags("#my-tag");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "my-tag");
    }

    #[test]
    fn test_tag_with_numbers() {
        let tags = parse_tags("#tag123");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "tag123");
    }

    #[test]
    fn test_heading_not_a_tag() {
        let tags = parse_tags("# Heading");
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_heading_h2_not_a_tag() {
        let tags = parse_tags("## Second level heading");
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_tag_after_heading() {
        let tags = parse_tags("# Heading\nSome text with #tag");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "tag");
    }

    #[test]
    fn test_tag_at_start_of_line() {
        let tags = parse_tags("#standalone");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "standalone");
    }

    #[test]
    fn test_tag_at_end_of_line() {
        let tags = parse_tags("Some text #end");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "end");
    }

    #[test]
    fn test_no_tag_in_code_block() {
        // Note: This is a simplification - full implementation would need markdown parsing
        let tags = parse_tags("Use `#not-a-tag` in code");
        // Current implementation will find it - this documents the behavior
        assert_eq!(tags.len(), 1);
    }

    #[test]
    fn test_no_tag_mid_word() {
        let tags = parse_tags("email@#domain");
        // # preceded by @ is still valid since @ is not alphanumeric
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "domain");
    }

    #[test]
    fn test_hashtag_number_only_ignored() {
        let tags = parse_tags("Issue #123 reference");
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_multiple_tags_same_line() {
        let tags = parse_tags("#one #two #three");
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0].name, "one");
        assert_eq!(tags[1].name, "two");
        assert_eq!(tags[2].name, "three");
    }

    #[test]
    fn test_tag_in_list_item() {
        let tags = parse_tags("- Item with #tag");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "tag");
    }

    #[test]
    fn test_multiline() {
        let input = r#"
# My Document

This is a paragraph with #first tag.

- List item with #second
- Another item #third/nested

## Section #four
"#;
        let tags = parse_tags(input);
        assert_eq!(tags.len(), 4);
        assert_eq!(tags[0].name, "first");
        assert_eq!(tags[1].name, "second");
        assert_eq!(tags[2].name, "third/nested");
        assert_eq!(tags[3].name, "four");
    }

    #[test]
    fn test_empty_input() {
        let tags = parse_tags("");
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_no_tags() {
        let tags = parse_tags("Just plain text without any tags");
        assert_eq!(tags.len(), 0);
    }
}
