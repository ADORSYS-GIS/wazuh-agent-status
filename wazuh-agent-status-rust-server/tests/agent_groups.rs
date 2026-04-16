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
