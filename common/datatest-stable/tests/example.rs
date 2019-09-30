use libra_datatest_stable::Result;
use std::{fs::File, io::Read, path::Path};

fn test_artifact(path: &Path) -> Result<()> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(())
}

libra_datatest_stable::harness!(test_artifact, "tests/files", r"^.*/*");
