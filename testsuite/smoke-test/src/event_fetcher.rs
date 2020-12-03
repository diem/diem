// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::test_utils::{diem_swarm_utils::get_diem_event_fetcher, setup_swarm_and_client_proxy};

#[test]
fn test_event_fetcher() {
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");

    let (env, mut client) = setup_swarm_and_client_proxy(1, 0);
    let events_fetcher = get_diem_event_fetcher(&env.validator_swarm, 0);

    client.create_next_account(false).unwrap();
    client.create_next_account(false).unwrap();
    client
        .mint_coins(&["mintb", "0", "100", "XUS"], true)
        .unwrap();

    client
        .mint_coins(&["mintb", "1", "100", "XUS"], true)
        .unwrap();

    client
        .transfer_coins(&["tb", "0", "1", "3", "XUS"], true)
        .unwrap();

    client
        .transfer_coins(&["tb", "0", "1", "4", "XUS"], true)
        .unwrap();

    let (account, _) = client.get_account_address_from_parameter("0").unwrap();
    let (sent_handle, received_handle) = runtime
        .block_on(events_fetcher.get_payment_event_handles(account))
        .unwrap()
        .unwrap();
    let mut sent_events = runtime
        .block_on(events_fetcher.get_all_events(&sent_handle))
        .unwrap();
    let received_events = runtime
        .block_on(events_fetcher.get_all_events(&received_handle))
        .unwrap();

    assert_eq!(received_events.len(), 1);
    assert_eq!(received_handle.count(), 1);
    assert_eq!(sent_handle.count(), 2);
    assert_eq!(sent_events.len(), 2);
    let evt1 = sent_events.pop().unwrap();
    let evt2 = sent_events.pop().unwrap();

    assert!(evt1.data.unwrap().amount.unwrap().amount == 4000000);
    assert!(evt2.data.unwrap().amount.unwrap().amount == 3000000);
}
