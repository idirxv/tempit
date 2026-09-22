//! Human-friendly rendering of the directory list.

use std::io::{self, Write};
use std::time::{Duration, SystemTime};

use anstyle::{AnsiColor, Style};

use crate::stats::DirStats;
use crate::store::TrackedDir;

const KIB: u64 = 1024;
const MIB: u64 = 1024 * KIB;
const DAY: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Align {
    Left,
    Right,
}

const COLUMNS: [(&str, Align); 6] = [
    ("#", Align::Right),
    ("LABEL", Align::Left),
    ("AGE", Align::Left),
    ("SIZE", Align::Right),
    ("CONTENTS", Align::Left),
    ("PATH", Align::Left),
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

/// Writes `rows` as an aligned table.
///
/// Styles are always emitted: write through an `anstream` stream to have them stripped when
/// the output is not a terminal or `NO_COLOR` is set.
pub fn write_table(
    out: &mut impl Write,
    rows: &[(TrackedDir, DirStats)],
    now: SystemTime,
) -> io::Result<()> {
    let header_style = Style::new().bold().underline();
    let header = COLUMNS.map(|(title, _)| Cell::new(title, header_style));
    let body: Vec<[Cell; 6]> = rows
        .iter()
        .map(|(dir, stats)| row_cells(dir, stats, now))
        .collect();

    let mut widths = header.each_ref().map(Cell::width);
    for cells in &body {
        for (width, cell) in widths.iter_mut().zip(cells) {
            *width = (*width).max(cell.width());
        }
    }

    for cells in std::iter::once(&header).chain(&body) {
        write_row(out, cells, &widths)?;
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

fn row_cells(dir: &TrackedDir, stats: &DirStats, now: SystemTime) -> [Cell; 6] {
    let color = |c: AnsiColor| Style::new().fg_color(Some(c.into()));

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

    [
        Cell::new(dir.name.id.to_string(), Style::new().bold()),
        label,
        age,
        Cell::new(human_size(stats.bytes), color(size_color)),
        Cell::new(
            format!(
                "{}, {}",
                plural(stats.files, "file"),
                plural(stats.dirs, "dir")
            ),
            Style::new(),
        ),
        Cell::new(dir.path.display().to_string(), Style::new()),
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

fn plural(count: u64, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
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
    fn table_is_aligned_without_trailing_spaces() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 86_400);
        let dir = |name: &str| TrackedDir {
            name: DirName::parse(name).unwrap(),
            path: PathBuf::from("/tmp/tempit-1000").join(name),
        };
        let rows = [
            (
                dir("2"),
                DirStats {
                    bytes: 0,
                    files: 1,
                    dirs: 0,
                    created: Some(now - Duration::from_secs(30)),
                },
            ),
            (
                dir("10-bugfix"),
                DirStats {
                    bytes: 2048,
                    files: 3,
                    dirs: 1,
                    created: Some(now - 2 * DAY),
                },
            ),
        ];

        let mut out = Vec::new();
        write_table(&mut out, &rows, now).unwrap();
        let plain = anstream::adapter::strip_str(std::str::from_utf8(&out).unwrap()).to_string();

        let expected = [
            " #  LABEL   AGE     SIZE  CONTENTS        PATH",
            " 2  -       now      0 B  1 file, 0 dirs  /tmp/tempit-1000/2",
            "10  bugfix  2d   2.0 KiB  3 files, 1 dir  /tmp/tempit-1000/10-bugfix",
        ];
        assert_eq!(plain.lines().collect::<Vec<_>>(), expected);
    }
}
