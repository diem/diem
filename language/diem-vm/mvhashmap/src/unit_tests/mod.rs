// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;

mod proptest_types;

#[test]
fn create_write_read_placeholder_struct() {
    let ap1 = b"/foo/b".to_vec();
    let ap2 = b"/foo/c".to_vec();
    let ap3 = b"/foo/d".to_vec();

    let data = vec![(ap1.clone(), 10), (ap2.clone(), 10), (ap2.clone(), 20)];

    let (mvtbl, max_dep) = MVHashMap::new_from(data);

    assert_eq!(2, max_dep);

    assert_eq!(2, mvtbl.len());

    // Reads that should go the the DB return Err(None)
    let r1 = mvtbl.read(&ap1, 5);
    assert_eq!(Err(None), r1);

    // Reads at a version return the previous versions, not this
    // version.
    let r1 = mvtbl.read(&ap1, 10);
    assert_eq!(Err(None), r1);

    // Check reads into non-ready structs return the Err(entry)

    // Reads at a higher version return the previous version
    let r1 = mvtbl.read(&ap1, 15);
    assert_eq!(Err(Some(10)), r1);

    // Writes populate the entry
    mvtbl.write(&ap1, 10, Some(vec![0, 0, 0])).unwrap();

    // Write to unexpected entries
    assert!(mvtbl.write(&ap1, 1, Some(vec![0, 0, 0])).is_err());
    assert!(mvtbl.write(&ap3, 10, Some(vec![0, 0, 0])).is_err());

    // Subsequent higher reads read this entry
    let r1 = mvtbl.read(&ap1, 15);
    assert_eq!(Ok(&Some(vec![0, 0, 0])), r1);

    // Set skip works
    assert!(mvtbl.skip(&ap1, 20).is_err());

    mvtbl.skip(&ap2, 20).unwrap();
    // Writes populate the entry
    mvtbl.write(&ap2, 10, Some(vec![0, 0, 0])).unwrap();

    // Higher reads skip this entry
    let r1 = mvtbl.read(&ap2, 25);
    assert_eq!(Ok(&Some(vec![0, 0, 0])), r1);
}
