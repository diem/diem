// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::epoch_ending::{
        backup::{EpochEndingBackupController, EpochEndingBackupOpt},
        restore::{
            EpochEndingRestoreController, EpochEndingRestoreOpt, EpochHistoryRestoreController,
        },
    },
    storage::{local_fs::LocalFs, BackupStorage},
    utils::{
        backup_service_client::BackupServiceClient, test_utils::tmp_db_with_random_content,
        ConcurrentDownloadsOpt, GlobalBackupOpt, GlobalRestoreOpt, RocksdbOpt, TrustedWaypointOpt,
    },
};
use backup_service::start_backup_service;
use diem_config::{config::RocksdbConfig, utils::get_available_port};
use diem_temppath::TempPath;
use diem_types::{
    ledger_info::LedgerInfoWithSignatures,
    proptest_types::{AccountInfoUniverse, LedgerInfoWithSignaturesGen},
    waypoint::Waypoint,
};
use diemdb::DiemDB;
use proptest::{collection::vec, prelude::*, std_facade::BTreeMap};
use std::{
    convert::TryInto,
    io::Write,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use storage_interface::DbReader;
use tokio::{runtime::Runtime, time::Duration};
use warp::Filter;

#[test]
fn end_to_end() {
    let (_src_db_dir, src_db, blocks) = tmp_db_with_random_content();
    let tgt_db_dir = TempPath::new();
    tgt_db_dir.create_as_dir().unwrap();

    let backup_dir = TempPath::new();
    backup_dir.create_as_dir().unwrap();
    let store: Arc<dyn BackupStorage> = Arc::new(LocalFs::new(backup_dir.path().to_path_buf()));

    let port = get_available_port();
    let rt = start_backup_service(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
        src_db,
    );
    let client = Arc::new(BackupServiceClient::new(format!(
        "http://localhost:{}",
        port
    )));

    let latest_epoch = blocks.last().unwrap().1.ledger_info().next_block_epoch();
    let target_version = blocks[blocks.len() / 2].1.ledger_info().version() + 1;
    let manifest_handle = rt
        .block_on(
            EpochEndingBackupController::new(
                EpochEndingBackupOpt {
                    start_epoch: 0,
                    end_epoch: latest_epoch,
                },
                GlobalBackupOpt {
                    max_chunk_size: 1024,
                },
                client,
                Arc::clone(&store),
            )
            .run(),
        )
        .unwrap();

    rt.block_on(
        EpochEndingRestoreController::new(
            EpochEndingRestoreOpt { manifest_handle },
            GlobalRestoreOpt {
                db_dir: Some(tgt_db_dir.path().to_path_buf()),
                dry_run: false,
                target_version: Some(target_version),
                trusted_waypoints: TrustedWaypointOpt::default(),
                rocksdb_opt: RocksdbOpt::default(),
                concurernt_downloads: ConcurrentDownloadsOpt::default(),
            }
            .try_into()
            .unwrap(),
            store,
        )
        .run(None),
    )
    .unwrap();

    let expected_ledger_infos = blocks
        .into_iter()
        .map(|(_, li)| li)
        .filter(|li| li.ledger_info().ends_epoch() && li.ledger_info().version() <= target_version)
        .collect::<Vec<_>>();
    let target_version_next_block_epoch = expected_ledger_infos
        .last()
        .map(|li| li.ledger_info().next_block_epoch())
        .unwrap_or(0);

    let tgt_db = DiemDB::open(
        &tgt_db_dir,
        true, /* read_only */
        None, /* pruner */
        RocksdbConfig::default(),
    )
    .unwrap();
    assert_eq!(
        tgt_db
            .get_epoch_ending_ledger_infos(0, target_version_next_block_epoch)
            .unwrap()
            .ledger_info_with_sigs,
        expected_ledger_infos,
    );

    rt.shutdown_timeout(Duration::from_secs(1));
}

prop_compose! {
    fn arb_epoch_endings_with_trusted_waypoints()(
        num_blocks in 1..10usize,
    )(
        mut universe in any_with::<AccountInfoUniverse>(3),
        blocks in vec(
            (
                1..100usize, // block size
                any::<LedgerInfoWithSignaturesGen>(),
                any::<bool>(), // overwrite validator set
                any::<bool>(), // trusted even if not overwriting validator set
            ),
            num_blocks
        )
    ) -> (Vec<LedgerInfoWithSignatures>, Vec<Waypoint>, bool) {
        let mut should_fail_without_waypoints = false;
        let mut res_lis = Vec::new();
        let mut res_waypoints = Vec::new();
        for (block_size, gen, overwrite, trusted) in blocks {
            let mut li = gen.materialize(&mut universe, block_size);
            if li.ledger_info().ends_epoch() {
                if overwrite && li.ledger_info().epoch() != 0 {
                    li = LedgerInfoWithSignatures::new(
                        li.ledger_info().clone(),
                        BTreeMap::new(),
                    );
                    should_fail_without_waypoints = true;
                }
                if overwrite || trusted {
                    res_waypoints.push(Waypoint::new_epoch_boundary(&li.ledger_info()).unwrap())
                }
                res_lis.push(li);

            }
        }
        (res_lis, res_waypoints, should_fail_without_waypoints)
    }
}

async fn mock_backup_service_get_epoch_ending_lis(lis: Vec<LedgerInfoWithSignatures>) -> u16 {
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
    let route = warp::path!("epoch_ending_ledger_infos" / usize / usize).map(move |start, end| {
        let mut response = Vec::<u8>::new();
        for li in &lis[start..end] {
            let bytes = bcs::to_bytes(&li).unwrap();
            let size_bytes = (bytes.len() as u32).to_be_bytes();
            response.write_all(&size_bytes).unwrap();
            response.write_all(&bytes).unwrap();
        }
        response
    });
    let (addr, svr) = warp::serve(route).bind_ephemeral(address);
    tokio::spawn(svr);
    addr.port()
}

async fn test_trusted_waypoints_impl(
    lis: Vec<LedgerInfoWithSignatures>,
    trusted_waypoints: Vec<Waypoint>,
    should_fail_without: bool,
) {
    let backup_dir = TempPath::new();
    backup_dir.create_as_dir().unwrap();
    let store: Arc<dyn BackupStorage> = Arc::new(LocalFs::new(backup_dir.path().to_path_buf()));
    let port = mock_backup_service_get_epoch_ending_lis(lis.clone()).await;
    let client = Arc::new(BackupServiceClient::new(format!(
        "http://localhost:{}",
        port
    )));

    let mut manifests = Vec::new();
    let mut start = 0;
    while start < lis.len() {
        let m = EpochEndingBackupController::new(
            EpochEndingBackupOpt {
                start_epoch: start as u64,
                end_epoch: std::cmp::min(start + 2, lis.len()) as u64,
            },
            GlobalBackupOpt {
                max_chunk_size: 1024,
            },
            client.clone(),
            Arc::clone(&store),
        )
        .run()
        .await
        .unwrap();
        manifests.push(m);
        start += 2;
    }

    let res_without_waypoints = EpochHistoryRestoreController::new(
        manifests.clone(),
        GlobalRestoreOpt {
            db_dir: None,
            dry_run: true,
            target_version: None,
            trusted_waypoints: TrustedWaypointOpt::default(),
            rocksdb_opt: RocksdbOpt::default(),
            concurernt_downloads: ConcurrentDownloadsOpt::default(),
        }
        .try_into()
        .unwrap(),
        Arc::clone(&store),
    )
    .run()
    .await;
    assert_eq!(should_fail_without, res_without_waypoints.is_err());

    let restored = EpochHistoryRestoreController::new(
        manifests,
        GlobalRestoreOpt {
            db_dir: None,
            dry_run: true,
            target_version: None,
            trusted_waypoints: TrustedWaypointOpt {
                trust_waypoint: trusted_waypoints,
            },
            rocksdb_opt: RocksdbOpt::default(),
            concurernt_downloads: ConcurrentDownloadsOpt::default(),
        }
        .try_into()
        .unwrap(),
        Arc::clone(&store),
    )
    .run()
    .await
    .unwrap();
    assert_eq!(
        lis.into_iter()
            .map(|li| li.ledger_info().clone())
            .collect::<Vec<_>>(),
        restored.epoch_endings,
    );
}

proptest! {
    #[test]
    fn trusted_waypoints(
        (lis, trusted_waypoints, should_fail_without) in arb_epoch_endings_with_trusted_waypoints()
    ) {
        Runtime::new().unwrap().block_on(test_trusted_waypoints_impl(lis, trusted_waypoints, should_fail_without))
    }
}
