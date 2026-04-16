//! Extracts Wazuh agent group names from the `merged.mg` policy file.

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use tracing::debug;

/// Parse the `merged.mg` file at `path` and return all group names found.
///
/// The file format contains:
/// - A leading comment line whose text is the primary group name.
/// - `# Source file: <group>/<filename> -->` markers for each included group.
pub fn extract_groups<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut groups = Vec::new();
    let mut first_comment_added = false;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l.trim().to_string(),
            Err(_) => continue,
        };

        if line.is_empty() {
            continue;
        }

        if !first_comment_added {
            if let Some(group) = extract_first_comment_group(&line) {
                groups.push(group);
                first_comment_added = true;
                continue;
            }
        }

        if let Some(group) = extract_source_file_group(&line) {
            groups.push(group);
        }
    }

    debug!(groups = ?groups, "Extracted agent groups");
    Ok(groups)
}

fn extract_first_comment_group(line: &str) -> Option<String> {
    if !line.starts_with('#') || line.contains("Source file:") {
        return None;
    }

    let candidate = line.trim_start_matches('#').trim().to_string();
    if candidate.is_empty() { None } else { Some(candidate) }
}

fn extract_source_file_group(line: &str) -> Option<String> {
    const MARKER: &str = "Source file:";

    let parts: Vec<&str> = line.splitn(2, MARKER).collect();
    if parts.len() != 2 {
        return None;
    }

    let mut path_part = parts[1].trim();
    if let Some(stripped) = path_part.strip_suffix("-->") {
        path_part = stripped.trim();
    }

    if let Some(idx) = path_part.find('/') {
        let group = &path_part[..idx];
        if !group.is_empty() {
            return Some(group.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_groups() -> io::Result<()> {
        let mut tmp_file = NamedTempFile::new()?;
        writeln!(tmp_file, "# main_group")?;
        writeln!(tmp_file, "some config data")?;
        writeln!(tmp_file, "# Source file: test_group/file.conf -->")?;
        writeln!(tmp_file, "more data")?;
        writeln!(tmp_file, "# Source file: another_group/other.conf -->")?;

        let groups = extract_groups(tmp_file.path())?;
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0], "main_group");
        assert_eq!(groups[1], "test_group");
        assert_eq!(groups[2], "another_group");

        Ok(())
    }

    #[test]
    fn test_extract_groups_no_secondary() -> io::Result<()> {
        let mut tmp_file = NamedTempFile::new()?;
        writeln!(tmp_file, "# only_one")?;
        writeln!(tmp_file, "no markers here")?;

        let groups = extract_groups(tmp_file.path())?;
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], "only_one");

        Ok(())
    }
}
