use std::io::Write;
use tempfile::NamedTempFile;
use wazuh_agent_status_rust_server::group_extractor::extract_groups;

#[test]
fn test_extract_groups_full() -> std::io::Result<()> {
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
fn test_extract_groups_single() -> std::io::Result<()> {
    let mut tmp_file = NamedTempFile::new()?;
    writeln!(tmp_file, "# only_one")?;
    writeln!(tmp_file, "no markers here")?;

    let groups = extract_groups(tmp_file.path())?;
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0], "only_one");

    Ok(())
}

#[test]
fn test_extract_groups_edge_cases() -> std::io::Result<()> {
    let mut tmp_file = NamedTempFile::new()?;
    // Empty comment
    writeln!(tmp_file, "#")?;
    writeln!(tmp_file, "#   ")?;
    // Actual first group
    writeln!(tmp_file, "# first_valid_group")?;
    // Empty lines
    writeln!(tmp_file, "")?;
    writeln!(tmp_file, "   ")?;
    // Source file with and without --> suffix, and without slash
    writeln!(tmp_file, "# Source file: test_no_arrow/conf.xml")?;
    writeln!(tmp_file, "# Source file: invalid_no_slash")?;
    writeln!(tmp_file, "# Source file: /leading_slash_empty_group")?;

    let groups = extract_groups(tmp_file.path())?;
    assert_eq!(groups, vec!["first_valid_group", "test_no_arrow"]);

    // Non-existent file
    assert!(extract_groups("/non/existent/path/merged.mg").is_err());

    Ok(())
}
