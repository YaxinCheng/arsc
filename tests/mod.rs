use arsc::{parse, write_to};
use std::io::Result;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

#[cfg(not(target_os = "windows"))]
const SAMPLE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/samples");

#[cfg(target_os = "windows")]
const SAMPLE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), r#"\tests\samples"#);

#[test]
fn test_read_write_matches() -> Result<()> {
    let entries = WalkDir::new(SAMPLE_PATH)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.map(DirEntry::into_path).ok())
        .filter(|path| {
            path.extension()
                .map(|extension| matches!(extension.to_str(), Some("arsc")))
                .unwrap_or_default()
        });
    for path in entries {
        let expected_bytes = std::fs::read(&path)?;
        let actual_bytes = read_then_write_to_bytes(&path)?;

        if actual_bytes.contains(&(char::REPLACEMENT_CHARACTER as u8)) {
            println!(
                "skipping test for {:?} because it contains char::REPLACEMENT_CHARACTER",
                path
            );
            continue;
        }
        assert_eq!(expected_bytes, actual_bytes)
    }
    Ok(())
}

fn read_then_write_to_bytes(path: &Path) -> Result<Vec<u8>> {
    let arsc = parse(path)?;
    let mut output = vec![];
    write_to(&arsc, &mut output)?;
    Ok(output)
}
