//! Human-friendly rendering of the directory list.

use std::fmt;
use std::io::{self, Write};
use std::path::Path;
use std::time::{Duration, SystemTime};

use anstyle::{AnsiColor, Style};

use crate::stats::DirStats;
use crate::store::TrackedDir;

const KIB: u64 = 1024;
const MIB: u64 = 1024 * KIB;
const DAY: Duration = Duration::from_secs(24 * 60 * 60);

/// Marks the directory containing the working directory.
const CURRENT_MARKER: &str = "▶";

/// One line of the listing.
pub struct Row {
    pub dir: TrackedDir,
    pub stats: DirStats,
    /// Whether the working directory is inside this directory.
    pub current: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Align {
    Left,
    Right,
}

const COLUMNS: [(&str, Align); 6] = [
    ("", Align::Left),
    ("#", Align::Right),
    ("LABEL", Align::Left),
    ("AGE", Align::Left),
    ("SIZE", Align::Right),
    ("CONTENTS", Align::Left),
];

struct Cell {
    text: String,
    style: Style,
}

impl Cell {
    fn new(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    fn width(&self) -> usize {
        self.text.chars().count()
    }
}

/// Writes `rows` as an aligned table, followed by a summary of their number, total size and
/// location.
///
/// Styles are always emitted: write through an `anstream` stream to have them stripped when
/// the output is not a terminal or `NO_COLOR` is set.
pub fn write_table(
    out: &mut impl Write,
    rows: &[Row],
    root: &Path,
    now: SystemTime,
) -> io::Result<()> {
    let header_style = Style::new().bold().underline();
    let header = COLUMNS.map(|(title, _)| Cell::new(title, header_style));
    let body: Vec<[Cell; 6]> = rows.iter().map(|row| row_cells(row, now)).collect();

    let mut widths = header.each_ref().map(Cell::width);
    // Keep room for the marker, so that the table does not shift when entering a directory.
    widths[0] = CURRENT_MARKER.chars().count();
    for cells in &body {
        for (width, cell) in widths.iter_mut().zip(cells) {
            *width = (*width).max(cell.width());
        }
    }

    for cells in std::iter::once(&header).chain(&body) {
        write_row(out, cells, &widths)?;
    }

    let dim = Style::new().dimmed();
    let total: u64 = rows.iter().map(|row| row.stats.bytes).sum();
    writeln!(
        out,
        "\n{dim}{}, {} in {}{dim:#}",
        plural(rows.len(), "directory", "directories"),
        human_size(total),
        root.display()
    )?;
    if rows.iter().any(|row| row.current) {
        writeln!(out, "{dim}({CURRENT_MARKER} = current directory){dim:#}")?;
    }
    Ok(())
}

fn write_row(out: &mut impl Write, cells: &[Cell; 6], widths: &[usize; 6]) -> io::Result<()> {
    for (index, ((cell, width), (_, align))) in cells.iter().zip(widths).zip(COLUMNS).enumerate() {
        let is_last = index == COLUMNS.len() - 1;
        let padding = width - cell.width();
        let (before, after) = match align {
            Align::Right => (padding, 0),
            // No trailing spaces at the end of the line.
            Align::Left if is_last => (0, 0),
            Align::Left => (0, padding),
        };
        let style = cell.style;
        write!(
            out,
            "{:before$}{style}{}{style:#}{:after$}",
            "", cell.text, ""
        )?;
        if !is_last {
            out.write_all(b"  ")?;
        }
    }
    writeln!(out)
}

fn row_cells(row: &Row, now: SystemTime) -> [Cell; 6] {
    let color = |c: AnsiColor| Style::new().fg_color(Some(c.into()));
    let Row {
        dir,
        stats,
        current,
    } = row;

    let marker = if *current {
        Cell::new(CURRENT_MARKER, color(AnsiColor::Cyan).bold())
    } else {
        Cell::new("", Style::new())
    };

    let label = match &dir.name.label {
        Some(label) => Cell::new(label.as_str(), color(AnsiColor::Cyan).bold()),
        None => Cell::new("-", Style::new().dimmed()),
    };

    let age = match stats.created {
        Some(created) => {
            let age = now.duration_since(created).unwrap_or_default();
            let style = color(if age >= DAY {
                AnsiColor::Yellow
            } else {
                AnsiColor::Green
            });
            Cell::new(human_age(age), style)
        }
        None => Cell::new("?", Style::new().dimmed()),
    };

    let size_color = match stats.bytes {
        b if b > 100 * MIB => AnsiColor::Red,
        b if b > 10 * MIB => AnsiColor::Yellow,
        _ => AnsiColor::Green,
    };

    let contents = match (stats.files, stats.dirs) {
        (0, 0) => Cell::new("empty", Style::new().dimmed()),
        (files, 0) => Cell::new(plural(files, "file", "files"), Style::new()),
        (0, dirs) => Cell::new(plural(dirs, "dir", "dirs"), Style::new()),
        (files, dirs) => Cell::new(
            format!(
                "{}, {}",
                plural(files, "file", "files"),
                plural(dirs, "dir", "dirs")
            ),
            Style::new(),
        ),
    };

    [
        marker,
        Cell::new(dir.name.id.to_string(), Style::new().bold()),
        label,
        age,
        Cell::new(human_size(stats.bytes), color(size_color)),
        contents,
    ]
}

/// Formats a byte count with binary units, e.g. `512 B` or `1.5 MiB`.
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["KiB", "MiB", "GiB", "TiB", "PiB"];
    if bytes < KIB {
        return format!("{bytes} B");
    }
    #[allow(clippy::cast_precision_loss)] // One decimal is displayed, precision loss is moot.
    let mut value = bytes as f64 / 1024.0;
    let mut unit = 0;
    // Switch units before the value would be displayed as "1024.0".
    while value >= 1023.95 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

/// Formats an age with its largest unit, e.g. `now`, `5m`, `3h` or `12d`.
pub fn human_age(age: Duration) -> String {
    match age.as_secs() {
        0..60 => "now".to_owned(),
        s @ 60..3600 => format!("{}m", s / 60),
        s @ 3600..86_400 => format!("{}h", s / 3600),
        s => format!("{}d", s / 86_400),
    }
}

/// Formats a count with the right form of its noun, e.g. `1 file` or `3 files`.
pub fn plural<N>(count: N, one: &str, many: &str) -> String
where
    N: fmt::Display + PartialEq + From<u8> + Copy,
{
    let noun = if count == N::from(1) { one } else { many };
    format!("{count} {noun}")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::name::DirName;

    #[test]
    fn sizes_use_binary_units() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(1023), "1023 B");
        assert_eq!(human_size(1024), "1.0 KiB");
        assert_eq!(human_size(1536), "1.5 KiB");
        assert_eq!(human_size(MIB - 1), "1.0 MiB");
        assert_eq!(human_size(10 * MIB), "10.0 MiB");
        assert_eq!(human_size(u64::MAX), "16384.0 PiB");
    }

    #[test]
    fn ages_use_their_largest_unit() {
        let secs = Duration::from_secs;
        assert_eq!(human_age(secs(0)), "now");
        assert_eq!(human_age(secs(59)), "now");
        assert_eq!(human_age(secs(60)), "1m");
        assert_eq!(human_age(secs(3599)), "59m");
        assert_eq!(human_age(secs(3600)), "1h");
        assert_eq!(human_age(secs(86_399)), "23h");
        assert_eq!(human_age(secs(86_400)), "1d");
        assert_eq!(human_age(secs(40 * 86_400)), "40d");
    }

    #[test]
    fn plurals() {
        assert_eq!(plural(0_u64, "file", "files"), "0 files");
        assert_eq!(plural(1_usize, "directory", "directories"), "1 directory");
        assert_eq!(plural(2_u64, "dir", "dirs"), "2 dirs");
    }

    fn row(name: &str, bytes: u64, files: u64, dirs: u64, created: SystemTime) -> Row {
        Row {
            dir: TrackedDir {
                name: DirName::parse(name).unwrap(),
                path: PathBuf::from("/tmp/tempit-1000").join(name),
            },
            stats: DirStats {
                bytes,
                files,
                dirs,
                created: Some(created),
            },
            current: false,
        }
    }

    fn render(rows: &[Row], now: SystemTime) -> Vec<String> {
        let mut out = Vec::new();
        write_table(&mut out, rows, Path::new("/tmp/tempit-1000"), now).unwrap();
        let text = std::str::from_utf8(&out).unwrap();
        let plain = anstream::adapter::strip_str(text).to_string();
        plain.lines().map(str::to_owned).collect()
    }

    #[test]
    fn table_is_compact_aligned_and_summarised() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 86_400);
        let rows = [
            row("2", 0, 0, 0, now - Duration::from_secs(30)),
            row("7", 12, 1, 0, now - Duration::from_secs(3 * 3600)),
            row("10-bugfix", 2048, 3, 1, now - 2 * DAY),
        ];

        let expected = [
            "    #  LABEL   AGE     SIZE  CONTENTS",
            "    2  -       now      0 B  empty",
            "    7  -       3h      12 B  1 file",
            "   10  bugfix  2d   2.0 KiB  3 files, 1 dir",
            "",
            "3 directories, 2.0 KiB in /tmp/tempit-1000",
        ];
        assert_eq!(render(&rows, now), expected);
    }

    #[test]
    fn current_directory_is_marked() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(86_400);
        let mut rows = [row("1", 0, 0, 2, now), row("2-api", 0, 0, 0, now)];
        rows[1].current = true;

        let expected = [
            "   #  LABEL  AGE  SIZE  CONTENTS",
            "   1  -      now   0 B  2 dirs",
            "▶  2  api    now   0 B  empty",
            "",
            "2 directories, 0 B in /tmp/tempit-1000",
            "(▶ = current directory)",
        ];
        assert_eq!(render(&rows, now), expected);
    }
}
