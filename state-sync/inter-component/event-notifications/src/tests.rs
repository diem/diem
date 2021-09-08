// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    Error, EventNotificationListener, EventNotificationSender, EventSubscriptionService,
    ReconfigNotificationListener,
};
use diem_infallible::RwLock;
use diem_types::{
    account_address::AccountAddress,
    contract_event::ContractEvent,
    event::EventKey,
    on_chain_config,
    on_chain_config::ON_CHAIN_CONFIG_REGISTRY,
    transaction::{Transaction, Version, WriteSetPayload},
};
use diem_vm::DiemVM;
use diemdb::DiemDB;
use executor_test_helpers::bootstrap_genesis;
use futures::{executor::block_on, FutureExt, StreamExt};
use move_core_types::language_storage::TypeTag;
use std::{convert::TryInto, sync::Arc};
use storage_interface::DbReaderWriter;

#[test]
fn test_all_configs_returned() {
    // Create subscription service and mock database
    let mut event_service = EventSubscriptionService::new(create_database());

    // Create reconfig subscribers
    let mut listener_1 = event_service.subscribe_to_reconfigurations().unwrap();
    let mut listener_2 = event_service.subscribe_to_reconfigurations().unwrap();
    let mut listener_3 = event_service.subscribe_to_reconfigurations().unwrap();

    // Notify the subscription service of 10 force reconfigurations and verify the
    // notifications contain all configs.
    let version = 0;
    let epoch = 1;
    let num_reconfigs = 10;
    for _ in 0..num_reconfigs {
        notify_initial_configs(&mut event_service, version);
        verify_reconfig_notifications_received(
            vec![&mut listener_1, &mut listener_2, &mut listener_3],
            version,
            epoch,
        );
    }

    // Notify the subscription service of 10 reconfiguration events and verify the
    // notifications contain all configs.
    let reconfig_event = create_test_event(on_chain_config::new_epoch_event_key());
    for _ in 0..num_reconfigs {
        notify_events(&mut event_service, version, vec![reconfig_event.clone()]);
        verify_reconfig_notifications_received(
            vec![&mut listener_1, &mut listener_2, &mut listener_3],
            version,
            epoch,
        );
    }
}

#[test]
fn test_reconfig_notification_no_queuing() {
    // Create subscription service and mock database
    let mut event_service = EventSubscriptionService::new(create_database());

    // Create reconfig subscribers
    let mut listener_1 = event_service.subscribe_to_reconfigurations().unwrap();
    let mut listener_2 = event_service.subscribe_to_reconfigurations().unwrap();

    // Notify the subscription service of 10 reconfiguration events
    let reconfig_event = create_test_event(on_chain_config::new_epoch_event_key());
    let num_reconfigs = 10;
    for _ in 0..num_reconfigs {
        notify_events(&mut event_service, 0, vec![reconfig_event.clone()]);
    }

    // Verify that only 1 notification was received by listener_1 (i.e., messages were dropped)
    let notification_count = count_reconfig_notifications(&mut listener_1);
    assert_eq!(notification_count, 1);

    // Notify the subscription service of 5 new force reconfigurations
    let num_reconfigs = 5;
    for _ in 0..num_reconfigs {
        notify_initial_configs(&mut event_service, 0);
    }

    // Verify that only 1 notification was received by listener_1 (i.e., messages were dropping)
    let notification_count = count_reconfig_notifications(&mut listener_1);
    assert_eq!(notification_count, 1);

    // Verify that only 1 notification was received by listener_2 (i.e., messages were dropped)
    let notification_count = count_reconfig_notifications(&mut listener_2);
    assert_eq!(notification_count, 1);
}

#[test]
fn test_event_and_reconfig_subscribers() {
    // Create subscription service and mock database
    let mut event_service = EventSubscriptionService::new(create_database());

    // Create several event keys
    let event_key_1 = create_random_event_key();
    let event_key_2 = create_random_event_key();
    let reconfig_event_key = on_chain_config::new_epoch_event_key();

    // Create subscribers for the various event keys
    let mut event_listener_1 = event_service
        .subscribe_to_events(vec![event_key_1])
        .unwrap();
    let mut event_listener_2 = event_service
        .subscribe_to_events(vec![event_key_1, event_key_2])
        .unwrap();
    let mut event_listener_3 = event_service
        .subscribe_to_events(vec![reconfig_event_key])
        .unwrap();

    // Create reconfiguration subscribers
    let mut reconfig_listener_1 = event_service.subscribe_to_reconfigurations().unwrap();
    let mut reconfig_listener_2 = event_service.subscribe_to_reconfigurations().unwrap();

    // Force an initial reconfiguration and verify the listeners received the notifications
    notify_initial_configs(&mut event_service, 0);
    verify_reconfig_notifications_received(
        vec![&mut reconfig_listener_1, &mut reconfig_listener_2],
        0,
        1,
    );
    verify_no_event_notifications(vec![
        &mut event_listener_1,
        &mut event_listener_2,
        &mut event_listener_3,
    ]);

    // Notify the service of a reconfiguration event and verify correct notifications
    let reconfig_event = create_test_event(reconfig_event_key);
    notify_events(&mut event_service, 0, vec![reconfig_event.clone()]);
    verify_reconfig_notifications_received(
        vec![&mut reconfig_listener_1, &mut reconfig_listener_2],
        0,
        1,
    );
    verify_event_notification_received(
        vec![&mut event_listener_3],
        0,
        vec![reconfig_event.clone()],
    );
    verify_no_event_notifications(vec![&mut event_listener_1, &mut event_listener_2]);

    // Notify the service of a reconfiguration and two other events, and verify notifications
    let event_1 = create_test_event(event_key_1);
    let event_2 = create_test_event(event_key_2);
    notify_events(
        &mut event_service,
        0,
        vec![event_1.clone(), event_2.clone(), reconfig_event.clone()],
    );
    verify_reconfig_notifications_received(
        vec![&mut reconfig_listener_1, &mut reconfig_listener_2],
        0,
        1,
    );
    verify_event_notification_received(vec![&mut event_listener_1], 0, vec![event_1.clone()]);
    verify_event_notification_received(
        vec![&mut event_listener_2],
        0,
        vec![event_1.clone(), event_2],
    );
    verify_event_notification_received(vec![&mut event_listener_3], 0, vec![reconfig_event]);

    // Notify the subscription service of one event only (no reconfigurations)
    notify_events(&mut event_service, 0, vec![event_1.clone()]);
    verify_event_notification_received(
        vec![&mut event_listener_1, &mut event_listener_2],
        0,
        vec![event_1],
    );
    verify_no_reconfig_notifications(vec![&mut reconfig_listener_1, &mut reconfig_listener_2]);
    verify_no_event_notifications(vec![&mut event_listener_3]);
}

#[test]
fn test_event_notification_queuing() {
    // Create subscription service and mock database
    let mut event_service = EventSubscriptionService::new(create_database());

    // Create several event keys
    let event_key_1 = create_random_event_key();
    let event_key_2 = on_chain_config::new_epoch_event_key();
    let event_key_3 = create_random_event_key();

    // Subscribe to the various events (except event_key_3)
    let mut listener_1 = event_service
        .subscribe_to_events(vec![event_key_1])
        .unwrap();
    let mut listener_2 = event_service
        .subscribe_to_events(vec![event_key_2])
        .unwrap();

    // Notify the subscription service of 1000 new events (with event_key_1)
    let num_events = 1000;
    let event_1 = create_test_event(event_key_1);
    for version in 0..num_events {
        notify_events(&mut event_service, version, vec![event_1.clone()]);
    }

    // Verify that less than 1000 notifications were received by listener_1
    // (i.e., messages were dropped).
    let notification_count = count_event_notifications_and_ensure_ordering(&mut listener_1);
    assert!(notification_count < num_events);

    // Notify the subscription service of 50 new events (with event_key_1 and event_key_2)
    let num_events = 50;
    let event_2 = create_test_event(event_key_1);
    for version in 0..num_events {
        notify_events(
            &mut event_service,
            version,
            vec![event_2.clone(), event_1.clone()],
        );
    }

    // Verify that 50 events were received (i.e., no message dropping occurred).
    let notification_count = count_event_notifications_and_ensure_ordering(&mut listener_1);
    assert_eq!(notification_count, num_events);

    // Notify the subscription service of 1000 new events (with event_key_3)
    let num_events = 1000;
    let event_3 = create_test_event(event_key_3);
    for version in 0..num_events {
        notify_events(&mut event_service, version, vec![event_3.clone()]);
    }

    // Verify neither listener received the event notifications
    verify_no_event_notifications(vec![&mut listener_1, &mut listener_2]);
}

#[test]
fn test_event_subscribers() {
    // Create subscription service and mock database
    let mut event_service = EventSubscriptionService::new(create_database());

    // Create several event keys
    let event_key_1 = create_random_event_key();
    let event_key_2 = create_random_event_key();
    let event_key_3 = on_chain_config::new_epoch_event_key();
    let event_key_4 = create_random_event_key();
    let event_key_5 = create_random_event_key();

    // Subscribe to the various events (except event_key_5)
    let mut listener_1 = event_service
        .subscribe_to_events(vec![event_key_1])
        .unwrap();
    let mut listener_2 = event_service
        .subscribe_to_events(vec![event_key_1, event_key_2])
        .unwrap();
    let mut listener_3 = event_service
        .subscribe_to_events(vec![event_key_2, event_key_3, event_key_4])
        .unwrap();

    // Notify the subscription service of a new event (with event_key_1)
    let version = 99;
    let event_1 = create_test_event(event_key_1);
    notify_events(&mut event_service, version, vec![event_1.clone()]);

    // Verify listener 1 and 2 receive the event notification
    verify_event_notification_received(
        vec![&mut listener_1, &mut listener_2],
        version,
        vec![event_1],
    );

    // Verify listener 3 doesn't get the event. Also verify that listener 1 and 2 don't get more events.
    verify_no_event_notifications(vec![&mut listener_1, &mut listener_2, &mut listener_3]);

    // Notify the subscription service of new events (with event_key_2 and event_key_3)
    let version = 200;
    let event_2 = create_test_event(event_key_2);
    let event_3 = create_test_event(event_key_3);
    notify_events(
        &mut event_service,
        version,
        vec![event_2.clone(), event_3.clone()],
    );

    // Verify listener 1 doesn't get any notifications
    verify_no_event_notifications(vec![&mut listener_1]);

    // Verify listener 2 and 3 get the event notifications
    verify_event_notification_received(vec![&mut listener_2], version, vec![event_2.clone()]);
    verify_event_notification_received(vec![&mut listener_3], version, vec![event_2, event_3]);

    // Notify the subscription service of a new event (with event_key_5)
    let event_5 = create_test_event(event_key_5);
    notify_events(&mut event_service, version, vec![event_5]);
    verify_no_event_notifications(vec![&mut listener_1]);
}

#[test]
fn test_no_events_no_subscribers() {
    // Create subscription service and mock database
    let mut event_service = EventSubscriptionService::new(create_database());

    // Verify a notification with zero events returns successfully
    notify_events(&mut event_service, 1, vec![]);

    // Verify no subscribers doesn't cause notification errors
    notify_events(
        &mut event_service,
        1,
        vec![create_test_event(create_random_event_key())],
    );

    // Attempt to subscribe to zero event keys
    assert!(matches!(
        event_service.subscribe_to_events(vec![]),
        Err(Error::CannotSubscribeToZeroEventKeys)
    ));

    // Add subscribers to the service
    let _event_listener = event_service.subscribe_to_events(vec![create_random_event_key()]);
    let _reconfig_listener = event_service.subscribe_to_reconfigurations();

    // Verify a notification with zero events returns successfully
    notify_events(&mut event_service, 1, vec![]);
}

// Counts the number of event notifications received by the listener. Also ensures that
// the notifications have increasing versions.
fn count_event_notifications_and_ensure_ordering(listener: &mut EventNotificationListener) -> u64 {
    let mut notification_received = true;
    let mut notification_count = 0;
    let mut last_version_received: i64 = -1;

    while notification_received {
        if let Some(event_notification) = listener.select_next_some().now_or_never() {
            notification_count += 1;
            assert!(last_version_received < event_notification.version.try_into().unwrap());
            last_version_received = event_notification.version.try_into().unwrap();
        } else {
            notification_received = false;
        }
    }

    notification_count
}

// Counts the number of reconfig notifications received by the listener.
fn count_reconfig_notifications(listener: &mut ReconfigNotificationListener) -> u64 {
    let mut notification_received = true;
    let mut notification_count = 0;

    while notification_received {
        if listener.select_next_some().now_or_never().is_some() {
            notification_count += 1;
        } else {
            notification_received = false;
        }
    }

    notification_count
}

// Ensures that no event notifications have been received by the listeners
fn verify_no_event_notifications(listeners: Vec<&mut EventNotificationListener>) {
    for listener in listeners {
        assert!(listener.select_next_some().now_or_never().is_none());
    }
}

// Ensures that no reconfig notifications have been received by the listeners
fn verify_no_reconfig_notifications(listeners: Vec<&mut ReconfigNotificationListener>) {
    for listener in listeners {
        assert!(listener.select_next_some().now_or_never().is_none());
    }
}

// Ensures that the specified listeners have received the expected notifications.
fn verify_event_notification_received(
    listeners: Vec<&mut EventNotificationListener>,
    expected_version: Version,
    expected_events: Vec<ContractEvent>,
) {
    for listener in listeners {
        if let Some(event_notification) = listener.select_next_some().now_or_never() {
            assert_eq!(event_notification.version, expected_version);
            assert_eq!(event_notification.subscribed_events, expected_events);
        } else {
            panic!("Expected an event notification but got None!");
        }
    }
}

// Ensures that the specified listeners have received the expected notifications.
// Also verifies that the reconfiguration notifications contain all on-chain configs.
fn verify_reconfig_notifications_received(
    listeners: Vec<&mut ReconfigNotificationListener>,
    expected_version: Version,
    expected_epoch: u64,
) {
    for listener in listeners {
        if let Some(reconfig_notification) = listener.select_next_some().now_or_never() {
            assert_eq!(reconfig_notification.version, expected_version);
            assert_eq!(
                reconfig_notification.on_chain_configs.epoch(),
                expected_epoch
            );

            let returned_configs = reconfig_notification.on_chain_configs.configs();
            for config in ON_CHAIN_CONFIG_REGISTRY {
                assert!(returned_configs.contains_key(config));
            }
        } else {
            panic!("Expected a reconfiguration notification but got None!");
        }
    }
}

fn notify_initial_configs(event_service: &mut EventSubscriptionService, version: Version) {
    block_on(event_service.notify_initial_configs(version)).unwrap();
}

fn notify_events(
    event_service: &mut EventSubscriptionService,
    version: Version,
    events: Vec<ContractEvent>,
) {
    block_on(event_service.notify_events(version, events)).unwrap();
}

fn create_test_event(event_key: EventKey) -> ContractEvent {
    ContractEvent::new(event_key, 0, TypeTag::Bool, bcs::to_bytes(&0).unwrap())
}

fn create_random_event_key() -> EventKey {
    EventKey::new_from_address(&AccountAddress::random(), 0)
}

fn create_database() -> Arc<RwLock<DbReaderWriter>> {
    // Generate a genesis change set
    let (genesis, _) = vm_genesis::test_genesis_change_set_and_validators(Some(1));

    // Create test diem database
    let db_path = diem_temppath::TempPath::new();
    db_path.create_as_dir().unwrap();
    let (_, db_rw) = DbReaderWriter::wrap(DiemDB::new_for_test(db_path.path()));

    // Bootstrap the genesis transaction
    let genesis_txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(genesis));
    bootstrap_genesis::<DiemVM>(&db_rw, &genesis_txn).unwrap();

    Arc::new(RwLock::new(db_rw))
}
