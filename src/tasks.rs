//! Parser for Obsidian Tasks plugin format.
//!
//! The Tasks plugin extends markdown checkboxes with emoji-based metadata for
//! due dates, scheduling, priorities, and recurrence.
//!
//! See the Tasks plugin documentation: <https://publish.obsidian.md/tasks/>

use chrono::NaiveDate;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

/// Task priority levels supported by the Tasks plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    /// ⏫ Highest priority
    Highest,
    /// 🔼 High priority
    High,
    /// 🔽 Low priority
    Low,
    /// ⏬ Lowest priority
    Lowest,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::Highest => write!(f, "⏫"),
            Priority::High => write!(f, "🔼"),
            Priority::Low => write!(f, "🔽"),
            Priority::Lowest => write!(f, "⏬"),
        }
    }
}

/// A parsed task from an Obsidian markdown file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// The task description with metadata stripped out.
    pub description: String,
    /// Whether the task is completed (marked with `[x]` or `[X]`).
    pub completed: bool,
    /// Due date from `📅 YYYY-MM-DD`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<NaiveDate>,
    /// Scheduled date from `⏳ YYYY-MM-DD`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_date: Option<NaiveDate>,
    /// Start date from `🛫 YYYY-MM-DD`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<NaiveDate>,
    /// Completion date from `✅ YYYY-MM-DD`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done_date: Option<NaiveDate>,
    /// Priority level from emoji markers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    /// Recurrence rule from `🔁 <pattern>`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence: Option<String>,
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let checkbox = if self.completed { "[x]" } else { "[ ]" };
        write!(f, "- {} {}", checkbox, self.description)?;

        if let Some(priority) = &self.priority {
            write!(f, " {}", priority)?;
        }
        if let Some(recurrence) = &self.recurrence {
            write!(f, " 🔁 {}", recurrence)?;
        }
        if let Some(start) = &self.start_date {
            write!(f, " 🛫 {}", start)?;
        }
        if let Some(scheduled) = &self.scheduled_date {
            write!(f, " ⏳ {}", scheduled)?;
        }
        if let Some(due) = &self.due_date {
            write!(f, " 📅 {}", due)?;
        }
        if let Some(done) = &self.done_date {
            write!(f, " ✅ {}", done)?;
        }

        Ok(())
    }
}

/// Extracts and parses all tasks from the given text.
///
/// Looks for lines matching the checkbox pattern `- [ ]` or `- [x]` and parses
/// any Tasks plugin metadata from them.
pub fn parse_tasks(input: &str) -> Vec<Task> {
    input
        .lines()
        .filter_map(|line| parse_task_line(line.trim_start()))
        .collect()
}

/// Finds and parses all tasks from markdown files at the given path.
///
/// If `path` is a file, parses tasks from that file.
/// If `path` is a directory, recursively walks it and parses all `.md` files.
/// Respects `.gitignore` and other ignore files.
pub fn find_tasks<P: AsRef<Path>>(path: P) -> io::Result<Vec<Task>> {
    let path = path.as_ref();
    let mut results = Vec::new();

    if path.is_file() {
        if path.extension().is_some_and(|ext| ext == "md") {
            let content = fs::read_to_string(path)?;
            results.extend(parse_tasks(&content));
        }
    } else if path.is_dir() {
        for entry in WalkBuilder::new(path).build() {
            let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let entry_path = entry.path();
            if entry_path.is_file() && entry_path.extension().is_some_and(|ext| ext == "md") {
                let content = fs::read_to_string(entry_path)?;
                results.extend(parse_tasks(&content));
            }
        }
    }

    Ok(results)
}

/// Attempts to parse a single line as a task.
///
/// Returns `None` if the line doesn't match the task checkbox pattern.
pub fn parse_task_line(line: &str) -> Option<Task> {
    // Match checkbox pattern: - [ ], - [x], - [X]
    let after_checkbox = if line.starts_with("- [ ] ") {
        Some((false, &line[6..]))
    } else if line.starts_with("- [x] ") || line.starts_with("- [X] ") {
        Some((true, &line[6..]))
    } else {
        None
    }?;

    let (completed, text) = after_checkbox;
    let mut description_parts: Vec<&str> = Vec::new();
    let mut due_date = None;
    let mut scheduled_date = None;
    let mut start_date = None;
    let mut done_date = None;
    let mut priority = None;
    let mut recurrence = None;

    let mut chars = text.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        match c {
            '📅' => {
                if let Some(date) = extract_date(text, i) {
                    due_date = Some(date);
                    // Skip past the date (space + 10 chars for YYYY-MM-DD)
                    skip_date(&mut chars);
                } else {
                    description_parts.push(&text[i..i + c.len_utf8()]);
                }
            }
            '⏳' => {
                if let Some(date) = extract_date(text, i) {
                    scheduled_date = Some(date);
                    skip_date(&mut chars);
                } else {
                    description_parts.push(&text[i..i + c.len_utf8()]);
                }
            }
            '🛫' => {
                if let Some(date) = extract_date(text, i) {
                    start_date = Some(date);
                    skip_date(&mut chars);
                } else {
                    description_parts.push(&text[i..i + c.len_utf8()]);
                }
            }
            '✅' => {
                if let Some(date) = extract_date(text, i) {
                    done_date = Some(date);
                    skip_date(&mut chars);
                } else {
                    description_parts.push(&text[i..i + c.len_utf8()]);
                }
            }
            '⏫' => {
                priority = Some(Priority::Highest);
            }
            '🔼' => {
                priority = Some(Priority::High);
            }
            '🔽' => {
                priority = Some(Priority::Low);
            }
            '⏬' => {
                priority = Some(Priority::Lowest);
            }
            '🔁' => {
                // Recurrence: extract everything until end of line or next emoji
                if let Some(rule) = extract_recurrence(text, i) {
                    recurrence = Some(rule);
                    // Skip to end since recurrence consumes the rest
                    break;
                } else {
                    description_parts.push(&text[i..i + c.len_utf8()]);
                }
            }
            _ => {
                // Regular character - find the extent of this text segment
                let start = i;
                let mut end = i + c.len_utf8();
                while let Some(&(j, next_c)) = chars.peek() {
                    if is_metadata_emoji(next_c) {
                        break;
                    }
                    end = j + next_c.len_utf8();
                    chars.next();
                }
                description_parts.push(&text[start..end]);
            }
        }
    }

    let description = description_parts.concat().trim().to_string();

    Some(Task {
        description,
        completed,
        due_date,
        scheduled_date,
        start_date,
        done_date,
        priority,
        recurrence,
    })
}

fn is_metadata_emoji(c: char) -> bool {
    matches!(c, '📅' | '⏳' | '🛫' | '✅' | '⏫' | '🔼' | '🔽' | '⏬' | '🔁')
}

fn extract_date(text: &str, emoji_pos: usize) -> Option<NaiveDate> {
    // Expect: emoji followed by space and YYYY-MM-DD
    let after_emoji = &text[emoji_pos..];
    let mut chars = after_emoji.chars();
    chars.next(); // skip emoji

    // Skip optional space
    let rest: String = chars.collect();
    let rest = rest.trim_start();

    if rest.len() >= 10 {
        let date_str = &rest[..10];
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
    } else {
        None
    }
}

fn skip_date(chars: &mut std::iter::Peekable<std::str::CharIndices>) {
    // Skip space and date characters (YYYY-MM-DD = 10 chars + optional space)
    let mut count = 0;
    while let Some(&(_, c)) = chars.peek() {
        if count > 11 || is_metadata_emoji(c) {
            break;
        }
        if c == ' ' || c == '-' || c.is_ascii_digit() {
            chars.next();
            count += 1;
        } else {
            break;
        }
    }
}

fn extract_recurrence(text: &str, emoji_pos: usize) -> Option<String> {
    let after_emoji = &text[emoji_pos..];
    let mut chars = after_emoji.chars();
    chars.next(); // skip emoji

    let rest: String = chars.collect();
    let rest = rest.trim_start();

    if rest.is_empty() {
        None
    } else {
        // Take until we hit another metadata emoji or end of string
        let end_pos = rest
            .char_indices()
            .find(|(_, c)| is_metadata_emoji(*c))
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        let rule = rest[..end_pos].trim().to_string();
        if rule.is_empty() {
            None
        } else {
            Some(rule)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_incomplete_task() {
        let task = parse_task_line("- [ ] Buy groceries").unwrap();
        assert_eq!(task.description, "Buy groceries");
        assert!(!task.completed);
        assert!(task.due_date.is_none());
    }

    #[test]
    fn test_simple_completed_task() {
        let task = parse_task_line("- [x] Buy groceries").unwrap();
        assert_eq!(task.description, "Buy groceries");
        assert!(task.completed);
    }

    #[test]
    fn test_completed_uppercase_x() {
        let task = parse_task_line("- [X] Buy groceries").unwrap();
        assert!(task.completed);
    }

    #[test]
    fn test_due_date() {
        let task = parse_task_line("- [ ] Buy groceries 📅 2024-01-15").unwrap();
        assert_eq!(task.description, "Buy groceries");
        assert_eq!(task.due_date, Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
    }

    #[test]
    fn test_scheduled_date() {
        let task = parse_task_line("- [ ] Call mom ⏳ 2024-02-20").unwrap();
        assert_eq!(task.description, "Call mom");
        assert_eq!(task.scheduled_date, Some(NaiveDate::from_ymd_opt(2024, 2, 20).unwrap()));
    }

    #[test]
    fn test_start_date() {
        let task = parse_task_line("- [ ] Start project 🛫 2024-03-01").unwrap();
        assert_eq!(task.description, "Start project");
        assert_eq!(task.start_date, Some(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()));
    }

    #[test]
    fn test_done_date() {
        let task = parse_task_line("- [x] Finished task ✅ 2024-01-10").unwrap();
        assert_eq!(task.description, "Finished task");
        assert!(task.completed);
        assert_eq!(task.done_date, Some(NaiveDate::from_ymd_opt(2024, 1, 10).unwrap()));
    }

    #[test]
    fn test_priority_highest() {
        let task = parse_task_line("- [ ] Urgent task ⏫").unwrap();
        assert_eq!(task.description, "Urgent task");
        assert_eq!(task.priority, Some(Priority::Highest));
    }

    #[test]
    fn test_priority_high() {
        let task = parse_task_line("- [ ] Important task 🔼").unwrap();
        assert_eq!(task.description, "Important task");
        assert_eq!(task.priority, Some(Priority::High));
    }

    #[test]
    fn test_priority_low() {
        let task = parse_task_line("- [ ] Low priority task 🔽").unwrap();
        assert_eq!(task.description, "Low priority task");
        assert_eq!(task.priority, Some(Priority::Low));
    }

    #[test]
    fn test_priority_lowest() {
        let task = parse_task_line("- [ ] Someday task ⏬").unwrap();
        assert_eq!(task.description, "Someday task");
        assert_eq!(task.priority, Some(Priority::Lowest));
    }

    #[test]
    fn test_recurrence() {
        let task = parse_task_line("- [ ] Weekly review 🔁 every week").unwrap();
        assert_eq!(task.description, "Weekly review");
        assert_eq!(task.recurrence, Some("every week".to_string()));
    }

    #[test]
    fn test_recurrence_complex() {
        let task = parse_task_line("- [ ] Pay rent 🔁 every month on the 1st").unwrap();
        assert_eq!(task.description, "Pay rent");
        assert_eq!(task.recurrence, Some("every month on the 1st".to_string()));
    }

    #[test]
    fn test_multiple_metadata() {
        let task = parse_task_line("- [ ] Project deadline ⏫ 📅 2024-06-30 🛫 2024-06-01").unwrap();
        assert_eq!(task.description, "Project deadline");
        assert_eq!(task.priority, Some(Priority::Highest));
        assert_eq!(task.due_date, Some(NaiveDate::from_ymd_opt(2024, 6, 30).unwrap()));
        assert_eq!(task.start_date, Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()));
    }

    #[test]
    fn test_all_metadata() {
        let task = parse_task_line(
            "- [x] Complete task 🛫 2024-01-01 ⏳ 2024-01-05 📅 2024-01-10 ✅ 2024-01-08 ⏫ 🔁 every week"
        ).unwrap();
        assert_eq!(task.description, "Complete task");
        assert!(task.completed);
        assert_eq!(task.start_date, Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
        assert_eq!(task.scheduled_date, Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()));
        assert_eq!(task.due_date, Some(NaiveDate::from_ymd_opt(2024, 1, 10).unwrap()));
        assert_eq!(task.done_date, Some(NaiveDate::from_ymd_opt(2024, 1, 8).unwrap()));
        assert_eq!(task.priority, Some(Priority::Highest));
        assert_eq!(task.recurrence, Some("every week".to_string()));
    }

    #[test]
    fn test_metadata_at_start() {
        let task = parse_task_line("- [ ] 📅 2024-01-15 Buy groceries").unwrap();
        assert_eq!(task.due_date, Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
        assert_eq!(task.description, "Buy groceries");
    }

    #[test]
    fn test_metadata_in_middle() {
        let task = parse_task_line("- [ ] Buy 📅 2024-01-15 groceries").unwrap();
        assert_eq!(task.due_date, Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
        assert_eq!(task.description, "Buy groceries");
    }

    #[test]
    fn test_not_a_task() {
        assert!(parse_task_line("Just regular text").is_none());
        assert!(parse_task_line("- Regular list item").is_none());
        assert!(parse_task_line("* [ ] Wrong bullet").is_none());
    }

    #[test]
    fn test_parse_tasks_multiple() {
        let input = r#"
# My Tasks

- [ ] First task 📅 2024-01-15
- [x] Second task ✅ 2024-01-10
- Regular list item
- [ ] Third task ⏫

Some other text
"#;
        let tasks = parse_tasks(input);
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].description, "First task");
        assert_eq!(tasks[1].description, "Second task");
        assert_eq!(tasks[2].description, "Third task");
    }

    #[test]
    fn test_indented_task() {
        let input = "    - [ ] Indented task 📅 2024-01-15";
        let tasks = parse_tasks(input);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].description, "Indented task");
        assert_eq!(tasks[0].due_date, Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
    }

    #[test]
    fn test_invalid_date_format() {
        let task = parse_task_line("- [ ] Task 📅 not-a-date").unwrap();
        assert!(task.due_date.is_none());
        // The emoji becomes part of description when date is invalid
        assert!(task.description.contains("📅"));
    }

    #[test]
    fn test_empty_task() {
        let task = parse_task_line("- [ ] ").unwrap();
        assert_eq!(task.description, "");
        assert!(!task.completed);
    }

    #[test]
    fn test_task_with_links() {
        let task = parse_task_line("- [ ] Review [[Project Plan]] 📅 2024-01-15").unwrap();
        assert_eq!(task.description, "Review [[Project Plan]]");
        assert_eq!(task.due_date, Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
    }

    #[test]
    fn test_task_with_tags() {
        let task = parse_task_line("- [ ] Fix bug #urgent #backend 📅 2024-01-15").unwrap();
        assert_eq!(task.description, "Fix bug #urgent #backend");
        assert_eq!(task.due_date, Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
    }

    #[test]
    fn test_display_simple_task() {
        let task = Task {
            description: "Buy groceries".to_string(),
            completed: false,
            due_date: None,
            scheduled_date: None,
            start_date: None,
            done_date: None,
            priority: None,
            recurrence: None,
        };
        assert_eq!(task.to_string(), "- [ ] Buy groceries");
    }

    #[test]
    fn test_display_completed_task() {
        let task = Task {
            description: "Done task".to_string(),
            completed: true,
            due_date: None,
            scheduled_date: None,
            start_date: None,
            done_date: None,
            priority: None,
            recurrence: None,
        };
        assert_eq!(task.to_string(), "- [x] Done task");
    }

    #[test]
    fn test_display_with_due_date() {
        let task = Task {
            description: "Task".to_string(),
            completed: false,
            due_date: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            scheduled_date: None,
            start_date: None,
            done_date: None,
            priority: None,
            recurrence: None,
        };
        assert_eq!(task.to_string(), "- [ ] Task 📅 2024-01-15");
    }

    #[test]
    fn test_display_with_priority() {
        let task = Task {
            description: "Urgent".to_string(),
            completed: false,
            due_date: None,
            scheduled_date: None,
            start_date: None,
            done_date: None,
            priority: Some(Priority::Highest),
            recurrence: None,
        };
        assert_eq!(task.to_string(), "- [ ] Urgent ⏫");
    }

    #[test]
    fn test_display_all_fields() {
        let task = Task {
            description: "Complete task".to_string(),
            completed: true,
            due_date: Some(NaiveDate::from_ymd_opt(2024, 1, 10).unwrap()),
            scheduled_date: Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
            start_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            done_date: Some(NaiveDate::from_ymd_opt(2024, 1, 8).unwrap()),
            priority: Some(Priority::Highest),
            recurrence: Some("every week".to_string()),
        };
        assert_eq!(
            task.to_string(),
            "- [x] Complete task ⏫ 🔁 every week 🛫 2024-01-01 ⏳ 2024-01-05 📅 2024-01-10 ✅ 2024-01-08"
        );
    }

    #[test]
    fn test_display_roundtrip() {
        let original = "- [ ] Project deadline ⏫ 📅 2024-06-30 🛫 2024-06-01";
        let task = parse_task_line(original).unwrap();
        let displayed = task.to_string();
        // Parse again and compare
        let reparsed = parse_task_line(&displayed).unwrap();
        assert_eq!(task.description, reparsed.description);
        assert_eq!(task.priority, reparsed.priority);
        assert_eq!(task.due_date, reparsed.due_date);
        assert_eq!(task.start_date, reparsed.start_date);
    }
}
