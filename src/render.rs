//! Idiomatic Rust table renderer using `comfy-table` and `owo-colors`.

use std::io::{self, Write};

use comfy_table::{Cell, ContentArrangement, Table, presets};
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
    if !use_color {
        table.set_width(220);
    }
    table.set_header(vec![
        "#", "Name", "Path", "Size", "Created", "Age", "Contents",
    ]);

    for (i, (info, stats)) in entries.iter().enumerate() {
        let idx = (i + 1).to_string();
        let created_str = info.created.format("%Y-%m-%d %H:%M").to_string();

        let (name_cell, size_cell, age_cell, contents_cell) = if use_color {
            let name = info.prefix.cyan().bold().to_string();
            let size = if stats.size_bytes > 100 * 1024 * 1024 {
                stats.human_size.red().to_string()
            } else if stats.size_bytes > 10 * 1024 * 1024 {
                stats.human_size.yellow().to_string()
            } else {
                stats.human_size.green().to_string()
            };
            let age = if stats.age.contains("day")
                || stats.age.contains("month")
                || stats.age.contains("year")
            {
                stats.age.yellow().to_string()
            } else {
                stats.age.green().to_string()
            };
            let contents = format!(
                "{} files, {} dirs",
                stats.file_count.blue(),
                stats.dir_count.blue()
            );
            (name, size, age, contents)
        } else {
            (
                info.prefix.clone(),
                stats.human_size.clone(),
                stats.age.clone(),
                format!("{} files, {} dirs", stats.file_count, stats.dir_count),
            )
        };

        table.add_row(vec![
            Cell::new(idx),
            Cell::new(name_cell),
            Cell::new(info.path.display().to_string()),
            Cell::new(size_cell),
            Cell::new(created_str),
            Cell::new(age_cell),
            Cell::new(contents_cell),
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
}
