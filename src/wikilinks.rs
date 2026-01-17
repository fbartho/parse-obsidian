//! Parser for Obsidian wikilinks.
//!
//! Obsidian uses `[[Page Name]]` syntax for internal links between notes,
//! with optional display text and heading anchors.

use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

/// A parsed wikilink from an Obsidian markdown file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wikilink {
    /// The target page name.
    pub target: String,
    /// Optional heading anchor (from `[[Page#Heading]]`).
    pub heading: Option<String>,
    /// Optional display text (from `[[Page|Display]]`).
    pub display: Option<String>,
}

impl fmt::Display for Wikilink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[[{}", self.target)?;
        if let Some(heading) = &self.heading {
            write!(f, "#{}", heading)?;
        }
        if let Some(display) = &self.display {
            write!(f, "|{}", display)?;
        }
        write!(f, "]]")
    }
}

/// Extracts and parses all wikilinks from the given text.
pub fn parse_wikilinks(input: &str) -> Vec<Wikilink> {
    let mut links = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Look for [[
        if i + 1 < bytes.len() && bytes[i] == b'[' && bytes[i + 1] == b'[' {
            let start = i + 2;
            // Find closing ]]
            if let Some(end) = find_closing_brackets(input, start) {
                let content = &input[start..end];
                if let Some(link) = parse_wikilink_content(content) {
                    links.push(link);
                }
                i = end + 2;
                continue;
            }
        }
        i += 1;
    }

    links
}

fn find_closing_brackets(input: &str, start: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut i = start;

    while i + 1 < bytes.len() {
        if bytes[i] == b']' && bytes[i + 1] == b']' {
            return Some(i);
        }
        i += 1;
    }

    None
}

fn parse_wikilink_content(content: &str) -> Option<Wikilink> {
    if content.is_empty() {
        return None;
    }

    // Split on | for display text
    let (link_part, display) = match content.find('|') {
        Some(pos) => {
            let display = content[pos + 1..].trim();
            let display = if display.is_empty() { None } else { Some(display.to_string()) };
            (&content[..pos], display)
        }
        None => (content, None),
    };

    // Split on # for heading
    let (target, heading) = match link_part.find('#') {
        Some(pos) => {
            let heading = link_part[pos + 1..].trim();
            let heading = if heading.is_empty() { None } else { Some(heading.to_string()) };
            (link_part[..pos].trim(), heading)
        }
        None => (link_part.trim(), None),
    };

    if target.is_empty() && heading.is_none() {
        return None;
    }

    Some(Wikilink {
        target: target.to_string(),
        heading,
        display,
    })
}

/// Finds and parses all wikilinks from markdown files at the given path.
///
/// If `path` is a file, parses wikilinks from that file.
/// If `path` is a directory, recursively walks it and parses all `.md` files.
/// Respects `.gitignore` and other ignore files.
pub fn find_wikilinks<P: AsRef<Path>>(path: P) -> io::Result<Vec<Wikilink>> {
    let path = path.as_ref();
    let mut results = Vec::new();

    if path.is_file() {
        if path.extension().is_some_and(|ext| ext == "md") {
            let content = fs::read_to_string(path)?;
            results.extend(parse_wikilinks(&content));
        }
    } else if path.is_dir() {
        for entry in WalkBuilder::new(path).build() {
            let entry = entry.map_err(io::Error::other)?;
            let entry_path = entry.path();
            if entry_path.is_file() && entry_path.extension().is_some_and(|ext| ext == "md") {
                let content = fs::read_to_string(entry_path)?;
                results.extend(parse_wikilinks(&content));
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_link() {
        let links = parse_wikilinks("Check out [[My Page]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "My Page");
        assert_eq!(links[0].heading, None);
        assert_eq!(links[0].display, None);
    }

    #[test]
    fn test_link_with_display_text() {
        let links = parse_wikilinks("See [[My Page|click here]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "My Page");
        assert_eq!(links[0].heading, None);
        assert_eq!(links[0].display, Some("click here".to_string()));
    }

    #[test]
    fn test_link_with_heading() {
        let links = parse_wikilinks("See [[My Page#Section]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "My Page");
        assert_eq!(links[0].heading, Some("Section".to_string()));
        assert_eq!(links[0].display, None);
    }

    #[test]
    fn test_link_with_heading_and_display() {
        let links = parse_wikilinks("See [[My Page#Section|the section]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "My Page");
        assert_eq!(links[0].heading, Some("Section".to_string()));
        assert_eq!(links[0].display, Some("the section".to_string()));
    }

    #[test]
    fn test_heading_only_link() {
        let links = parse_wikilinks("See [[#Section]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "");
        assert_eq!(links[0].heading, Some("Section".to_string()));
    }

    #[test]
    fn test_multiple_links() {
        let links = parse_wikilinks("Link to [[Page One]] and [[Page Two]]");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].target, "Page One");
        assert_eq!(links[1].target, "Page Two");
    }

    #[test]
    fn test_link_with_path() {
        let links = parse_wikilinks("See [[folder/My Page]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "folder/My Page");
    }

    #[test]
    fn test_link_at_start() {
        let links = parse_wikilinks("[[Start]] of line");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Start");
    }

    #[test]
    fn test_link_at_end() {
        let links = parse_wikilinks("End of line [[End]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "End");
    }

    #[test]
    fn test_empty_link_ignored() {
        let links = parse_wikilinks("Empty [[]] link");
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn test_single_bracket_not_link() {
        let links = parse_wikilinks("Array [index] access");
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn test_unclosed_link() {
        let links = parse_wikilinks("Unclosed [[link");
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn test_multiline() {
        let input = r#"
# Document

See [[Page One]] for details.

Also check:
- [[Page Two]]
- [[Page Three#Section|the section]]
"#;
        let links = parse_wikilinks(input);
        assert_eq!(links.len(), 3);
        assert_eq!(links[0].target, "Page One");
        assert_eq!(links[1].target, "Page Two");
        assert_eq!(links[2].target, "Page Three");
        assert_eq!(links[2].heading, Some("Section".to_string()));
        assert_eq!(links[2].display, Some("the section".to_string()));
    }

    #[test]
    fn test_adjacent_links() {
        let links = parse_wikilinks("[[One]][[Two]]");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].target, "One");
        assert_eq!(links[1].target, "Two");
    }

    #[test]
    fn test_link_with_special_chars() {
        let links = parse_wikilinks("[[Page (2024)]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Page (2024)");
    }

    #[test]
    fn test_link_with_emoji() {
        let links = parse_wikilinks("[[📝 Notes]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "📝 Notes");
    }

    #[test]
    fn test_not_embed() {
        // Embeds start with ! - this tests we don't confuse them
        let links = parse_wikilinks("![[Image.png]]");
        // We still parse it as a wikilink - embed detection is separate
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Image.png");
    }

    #[test]
    fn test_whitespace_trimmed() {
        let links = parse_wikilinks("[[ Page Name ]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Page Name");
    }

    #[test]
    fn test_pipe_only() {
        let links = parse_wikilinks("[[Page|]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Page");
        assert_eq!(links[0].display, None);
    }

    #[test]
    fn test_hash_only() {
        let links = parse_wikilinks("[[Page#]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Page");
        assert_eq!(links[0].heading, None);
    }
}
