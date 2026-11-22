//! Idiomatic Rust table renderer using `comfy-table` and `owo-colors`.

use std::io::{self, Write};

use comfy_table::{Attribute, Cell, CellAlignment, Color, ContentArrangement, Table, presets};
use owo_colors::OwoColorize;

use crate::models::{DirectoryInfo, DirectoryStats};

/// Render a list of `(DirectoryInfo, DirectoryStats)` pairs to `writer`.
pub fn render_entries_to<W: Write>(
    writer: &mut W,
    entries: &[(DirectoryInfo, DirectoryStats)],
    title: &str,
    use_color: bool,
) -> io::Result<()> {
    if entries.is_empty() {
        if use_color {
            writeln!(writer, "{}", "No temporary directories found.".yellow())?;
        } else {
            writeln!(writer, "No temporary directories found.")?;
        }
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);
    if use_color {
        table.enforce_styling();
    } else {
        table.set_width(220);
    }
    table.set_header(vec![
        "#", "Name", "Path", "Size", "Created", "Age", "Contents",
    ]);

    for (i, (info, stats)) in entries.iter().enumerate() {
        let idx_cell = Cell::new((i + 1).to_string()).set_alignment(CellAlignment::Right);
        let created_str = info.created.format("%Y-%m-%d %H:%M").to_string();
        let contents_str = format!("{} files, {} dirs", stats.file_count, stats.dir_count);

        let mut name_cell = Cell::new(&info.prefix);
        let mut size_cell = Cell::new(&stats.human_size).set_alignment(CellAlignment::Right);
        let mut age_cell = Cell::new(&stats.age);
        let mut contents_cell = Cell::new(contents_str);

        if use_color {
            name_cell = name_cell.fg(Color::Cyan).add_attribute(Attribute::Bold);

            let size_color = if stats.size_bytes > 100 * 1024 * 1024 {
                Color::Red
            } else if stats.size_bytes > 10 * 1024 * 1024 {
                Color::Yellow
            } else {
                Color::Green
            };
            size_cell = size_cell.fg(size_color);

            let age_color = if stats.age.contains("day")
                || stats.age.contains("month")
                || stats.age.contains("year")
            {
                Color::Yellow
            } else {
                Color::Green
            };
            age_cell = age_cell.fg(age_color);

            contents_cell = contents_cell.fg(Color::Blue);
        }

        table.add_row(vec![
            idx_cell,
            name_cell,
            Cell::new(info.path.display().to_string()),
            size_cell,
            Cell::new(created_str),
            age_cell,
            contents_cell,
        ]);
    }

    writeln!(writer)?;
    writeln!(writer, "{title}")?;
    writeln!(writer, "{table}")?;
    writeln!(writer)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::NaiveDateTime;

    use super::*;

    fn strip_ansi(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' && chars.peek() == Some(&'[') {
                chars.next();
                for next in chars.by_ref() {
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    #[test]
    fn render_empty_and_non_empty() {
        let mut out = Vec::new();
        render_entries_to(&mut out, &[], "Temporary Directories", false).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap().trim(),
            "No temporary directories found."
        );

        let info = DirectoryInfo {
            path: PathBuf::from("/tmp/demo_1234abcd"),
            created: NaiveDateTime::parse_from_str("2026-11-21T12:00:00", "%Y-%m-%dT%H:%M:%S")
                .unwrap(),
            prefix: "demo".to_string(),
        };
        let stats = DirectoryStats {
            size_bytes: 1024,
            human_size: "1.0 KiB".to_string(),
            file_count: 2,
            dir_count: 1,
            age: "now".to_string(),
        };
        let mut out2 = Vec::new();
        render_entries_to(&mut out2, &[(info, stats)], "Temporary Directories", false).unwrap();
        let text = String::from_utf8(out2).unwrap();
        assert!(text.contains("/tmp/demo_1234abcd"));
        assert!(text.contains("1.0 KiB"));
        assert!(text.contains("2 files, 1 dirs"));
    }

    #[test]
    fn colored_table_preserves_column_alignment() {
        let entries = vec![
            (
                DirectoryInfo {
                    path: PathBuf::from("/tmp/short_1234"),
                    created: NaiveDateTime::parse_from_str(
                        "2026-11-21T12:00:00",
                        "%Y-%m-%dT%H:%M:%S",
                    )
                    .unwrap(),
                    prefix: "a".to_string(),
                },
                DirectoryStats {
                    size_bytes: 512,
                    human_size: "512 Bytes".to_string(),
                    file_count: 1,
                    dir_count: 0,
                    age: "now".to_string(),
                },
            ),
            (
                DirectoryInfo {
                    path: PathBuf::from("/tmp/longer_prefix_abcd5678"),
                    created: NaiveDateTime::parse_from_str(
                        "2026-11-18T09:30:00",
                        "%Y-%m-%dT%H:%M:%S",
                    )
                    .unwrap(),
                    prefix: "longer_prefix".to_string(),
                },
                DirectoryStats {
                    size_bytes: 150 * 1024 * 1024,
                    human_size: "150.0 MiB".to_string(),
                    file_count: 120,
                    dir_count: 14,
                    age: "3 days ago".to_string(),
                },
            ),
        ];

        let mut colored_out = Vec::new();
        render_entries_to(&mut colored_out, &entries, "Temporary Directories", true).unwrap();
        let colored = String::from_utf8(colored_out).unwrap();
        assert!(
            colored.contains("\x1b["),
            "expected ANSI color codes when use_color = true"
        );

        let stripped = strip_ansi(&colored);
        let table_lines: Vec<&str> = stripped
            .lines()
            .filter(|line| {
                line.starts_with('┌')
                    || line.starts_with('│')
                    || line.starts_with('├')
                    || line.starts_with('└')
            })
            .collect();
        assert!(!table_lines.is_empty());

        let expected_width = table_lines[0].chars().count();
        for line in &table_lines {
            assert_eq!(
                line.chars().count(),
                expected_width,
                "misaligned table line: {line:?}"
            );
        }

        let pipe_positions: Vec<Vec<usize>> = table_lines
            .iter()
            .filter(|line| line.starts_with('│'))
            .map(|line| {
                line.chars()
                    .enumerate()
                    .filter_map(|(idx, ch)| (ch == '│').then_some(idx))
                    .collect()
            })
            .collect();
        assert!(pipe_positions.len() >= 3);
        for positions in &pipe_positions[1..] {
            assert_eq!(
                positions, &pipe_positions[0],
                "column separator positions differ across rows"
            );
        }
    }
}
