// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    peer::DisconnectReason,
    peer_manager::{conn_notifs_channel, ConnectionRequest},
    transport::ConnectionMetadata,
};
use channel::{diem_channel, message_queues::QueueStyle};
use diem_config::config::{Peer, PeerRole, PeerSet, HANDSHAKE_VERSION};
use diem_crypto::{test_utils::TEST_SEED, x25519, Uniform};
use diem_logger::info;
use diem_time_service::{MockTimeService, TimeService};
use diem_types::network_address::NetworkAddress;
use futures::{executor::block_on, future, SinkExt};
use maplit::{hashmap, hashset};
use rand::rngs::StdRng;
use std::{io, str::FromStr};
use tokio_retry::strategy::FixedInterval;

const MAX_TEST_CONNECTIONS: usize = 3;
const CONNECTIVITY_CHECK_INTERVAL: Duration = Duration::from_secs(5);
const CONNECTION_DELAY: Duration = Duration::from_millis(100);
const MAX_CONNECTION_DELAY: Duration = Duration::from_secs(60);
const DEFAULT_BASE_ADDR: &str = "/ip4/127.0.0.1/tcp/9090";

// TODO(philiphayes): just use `CONNECTION_DELAY + MAX_CONNNECTION_DELAY_JITTER`
// when the const adds are stabilized, instead of this weird thing...
const MAX_DELAY_WITH_JITTER: Duration = Duration::from_millis(
    CONNECTION_DELAY.as_millis() as u64 + MAX_CONNECTION_DELAY_JITTER.as_millis() as u64,
);

fn network_address(addr_str: &'static str) -> NetworkAddress {
    NetworkAddress::from_str(addr_str).unwrap()
}
fn network_address_with_pubkey(
    addr_str: &'static str,
    pubkey: x25519::PublicKey,
) -> NetworkAddress {
    network_address(addr_str).append_prod_protos(pubkey, HANDSHAKE_VERSION)
}

fn peer_id(index: usize) -> PeerId {
    PeerId::new((index as u128).to_be_bytes())
}

fn test_peer(index: usize) -> (PeerId, Peer, x25519::PublicKey, NetworkAddress) {
    test_peer_with_address(index, DEFAULT_BASE_ADDR)
}

fn test_peer_with_address(
    index: usize,
    addr_str: &'static str,
) -> (PeerId, Peer, x25519::PublicKey, NetworkAddress) {
    let peer_id = peer_id(index);
    let pubkey = x25519::PrivateKey::generate_for_testing().public_key();
    let pubkeys = hashset! { pubkey };
    let addr = network_address_with_pubkey(addr_str, pubkey);
    (
        peer_id,
        Peer::new(vec![addr.clone()], pubkeys, PeerRole::Validator),
        pubkey,
        addr,
    )
}

fn update_peer_with_address(mut peer: Peer, addr_str: &'static str) -> (Peer, NetworkAddress) {
    let keys: Vec<_> = peer.keys.iter().collect();
    let key = *keys.first().unwrap();
    let addr = network_address_with_pubkey(addr_str, *key);
    peer.addresses = vec![addr.clone()];
    (peer, addr)
}

struct TestHarness {
    trusted_peers: Arc<RwLock<PeerSet>>,
    mock_time: MockTimeService,
    connection_reqs_rx: diem_channel::Receiver<PeerId, ConnectionRequest>,
    connection_notifs_tx: conn_notifs_channel::Sender,
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
}

impl TestHarness {
    fn new(seeds: PeerSet) -> (Self, ConnectivityManager<FixedInterval>) {
        let network_context = NetworkContext::mock();
        let time_service = TimeService::mock();
        let (connection_reqs_tx, connection_reqs_rx) = diem_channel::new(QueueStyle::FIFO, 1, None);
        let (connection_notifs_tx, connection_notifs_rx) = conn_notifs_channel::new();
        let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(0);
        let trusted_peers = Arc::new(RwLock::new(HashMap::new()));

        let conn_mgr = ConnectivityManager::new(
            network_context,
            time_service.clone(),
            trusted_peers.clone(),
            seeds,
            ConnectionRequestSender::new(connection_reqs_tx),
            connection_notifs_rx,
            conn_mgr_reqs_rx,
            CONNECTIVITY_CHECK_INTERVAL,
            FixedInterval::new(CONNECTION_DELAY),
            MAX_CONNECTION_DELAY,
            Some(MAX_TEST_CONNECTIONS),
            true, /* mutual_authentication */
        );
        let mock = Self {
            trusted_peers,
            mock_time: time_service.into_mock(),
            connection_reqs_rx,
            connection_notifs_tx,
            conn_mgr_reqs_tx,
        };
        (mock, conn_mgr)
    }

    async fn trigger_connectivity_check(&self) {
        info!("Advance time to trigger connectivity check");
        self.mock_time
            .advance_async(CONNECTIVITY_CHECK_INTERVAL)
            .await;
    }

    async fn trigger_pending_dials(&self) {
        info!("Advance time to trigger dial");
        self.mock_time.advance_async(MAX_DELAY_WITH_JITTER).await;
    }

    async fn get_connected_size(&mut self) -> usize {
        info!("Sending ConnectivityRequest::GetConnectedSize");
        let (queue_size_tx, queue_size_rx) = oneshot::channel();
        self.conn_mgr_reqs_tx
            .send(ConnectivityRequest::GetConnectedSize(queue_size_tx))
            .await
            .unwrap();
        queue_size_rx.await.unwrap()
    }

    async fn get_dial_queue_size(&mut self) -> usize {
        info!("Sending ConnectivityRequest::GetDialQueueSize");
        let (queue_size_tx, queue_size_rx) = oneshot::channel();
        self.conn_mgr_reqs_tx
            .send(ConnectivityRequest::GetDialQueueSize(queue_size_tx))
            .await
            .unwrap();
        queue_size_rx.await.unwrap()
    }

    async fn send_new_peer_await_delivery(
        &mut self,
        peer_id: PeerId,
        notif_peer_id: PeerId,
        address: NetworkAddress,
    ) {
        info!(
            "Sending NewPeer notification for peer: {}",
            peer_id.short_str()
        );
        let mut metadata = ConnectionMetadata::mock_with_role_and_origin(
            notif_peer_id,
            PeerRole::Unknown,
            ConnectionOrigin::Outbound,
        );
        metadata.addr = address;
        let notif = peer_manager::ConnectionNotification::NewPeer(metadata, NetworkContext::mock());
        self.send_notification_await_delivery(peer_id, notif).await;
    }

    async fn send_lost_peer_await_delivery(&mut self, peer_id: PeerId, address: NetworkAddress) {
        info!(
            "Sending LostPeer notification for peer: {}",
            peer_id.short_str()
        );
        let mut metadata = ConnectionMetadata::mock_with_role_and_origin(
            peer_id,
            PeerRole::Unknown,
            ConnectionOrigin::Outbound,
        );
        metadata.addr = address;
        let notif = peer_manager::ConnectionNotification::LostPeer(
            metadata,
            NetworkContext::mock(),
            DisconnectReason::ConnectionLost,
        );
        self.send_notification_await_delivery(peer_id, notif).await;
    }

    async fn send_notification_await_delivery(
        &mut self,
        peer_id: PeerId,
        notif: peer_manager::ConnectionNotification,
    ) {
        let (delivered_tx, delivered_rx) = oneshot::channel();
        self.connection_notifs_tx
            .push_with_feedback(peer_id, notif, Some(delivered_tx))
            .unwrap();
        delivered_rx.await.unwrap();
    }

    async fn expect_disconnect_inner(
        &mut self,
        peer_id: PeerId,
        address: NetworkAddress,
        result: Result<(), PeerManagerError>,
    ) {
        info!("Waiting to receive disconnect request");
        let success = result.is_ok();
        match self.connection_reqs_rx.next().await.unwrap() {
            ConnectionRequest::DisconnectPeer(p, result_tx) => {
                assert_eq!(peer_id, p);
                result_tx.send(result).unwrap();
            }
            request => panic!(
                "Unexpected ConnectionRequest, expected DisconnectPeer: {:?}",
                request
            ),
        }
        if success {
            self.send_lost_peer_await_delivery(peer_id, address).await;
        }
    }

    async fn expect_disconnect_success(&mut self, peer_id: PeerId, address: NetworkAddress) {
        self.expect_disconnect_inner(peer_id, address, Ok(())).await;
    }

    async fn expect_disconnect_fail(&mut self, peer_id: PeerId, address: NetworkAddress) {
        let error = PeerManagerError::NotConnected(peer_id);
        self.expect_disconnect_inner(peer_id, address, Err(error))
            .await;
    }

    async fn wait_until_empty_dial_queue(&mut self) {
        // Wait for dial queue to be empty. Without this, it's impossible to guarantee that a completed
        // dial is removed from a dial queue. We need this guarantee to see the effects of future
        // triggers for connectivity check.
        info!("Waiting for dial queue to be empty");
        while self.get_dial_queue_size().await > 0 {}
    }

    async fn expect_one_dial_inner(
        &mut self,
        result: Result<(), PeerManagerError>,
    ) -> (PeerId, NetworkAddress) {
        info!("Waiting to receive dial request");
        let success = result.is_ok();
        let (peer_id, address) = match self.connection_reqs_rx.next().await.unwrap() {
            ConnectionRequest::DialPeer(peer_id, address, result_tx) => {
                result_tx.send(result).unwrap();
                (peer_id, address)
            }
            request => panic!(
                "Unexpected ConnectionRequest, expected DialPeer: {:?}",
                request
            ),
        };
        if success {
            self.send_new_peer_await_delivery(peer_id, peer_id, address.clone())
                .await;
        }
        (peer_id, address)
    }

    async fn expect_one_dial(
        &mut self,
        expected_peer_id: PeerId,
        expected_address: NetworkAddress,
        result: Result<(), PeerManagerError>,
    ) {
        let (peer_id, address) = self.expect_one_dial_inner(result).await;

        assert_eq!(peer_id, expected_peer_id);
        assert_eq!(address, expected_address);

        self.wait_until_empty_dial_queue().await;
    }

    async fn expect_one_dial_success(
        &mut self,
        expected_peer_id: PeerId,
        expected_address: NetworkAddress,
    ) {
        self.expect_one_dial(expected_peer_id, expected_address, Ok(()))
            .await;
    }

    async fn expect_one_dial_fail(
        &mut self,
        expected_peer_id: PeerId,
        expected_address: NetworkAddress,
    ) {
        let error = PeerManagerError::IoError(io::Error::from(io::ErrorKind::ConnectionRefused));
        self.expect_one_dial(expected_peer_id, expected_address, Err(error))
            .await;
    }

    async fn expect_num_dials(&mut self, num_expected: usize) {
        for _ in 0..num_expected {
            let _ = self.expect_one_dial_inner(Ok(())).await;
        }
        self.wait_until_empty_dial_queue().await;
    }

    async fn send_update_discovered_peers(&mut self, src: DiscoverySource, peers: PeerSet) {
        info!("Sending UpdateDiscoveredPeers");
        self.conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateDiscoveredPeers(src, peers))
            .await
            .unwrap();
    }
}

#[test]
fn connect_to_seeds_on_startup() {
    let (seed_peer_id, seed_peer, _, seed_addr) = test_peer(1);
    let seeds = hashmap! {seed_peer_id => seed_peer.clone()};
    let (mut mock, conn_mgr) = TestHarness::new(seeds.clone());

    let test = async move {
        // Peer manager receives a request to connect to the other peer.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(seed_peer_id, seed_addr.clone())
            .await;

        // Sending an UpdateDiscoveredPeers with the same seed address should not
        // trigger any dials.
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, seeds)
            .await;
        mock.trigger_connectivity_check().await;
        assert_eq!(0, mock.get_dial_queue_size().await);

        // Sending new address of seed peer
        let (new_seed, new_seed_addr) =
            update_peer_with_address(seed_peer, "/ip4/127.0.1.1/tcp/8080");
        let update = hashmap! {seed_peer_id => new_seed};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, update)
            .await;

        // We expect the peer which changed its address to also disconnect.
        mock.send_lost_peer_await_delivery(seed_peer_id, seed_addr.clone())
            .await;

        // We should try to connect to both the new address and seed address.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_fail(seed_peer_id, new_seed_addr).await;

        // Waiting to receive dial request to seed peer at seed address
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(seed_peer_id, seed_addr).await;
    };
    block_on(future::join(conn_mgr.start(), test));
}

#[test]
fn addr_change() {
    let (other_peer_id, other_peer, _, other_addr) = test_peer(0);
    let (mut mock, conn_mgr) = TestHarness::new(HashMap::new());

    let test = async move {
        // Sending address of other peer
        let update = hashmap! {other_peer_id => other_peer.clone()};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, update.clone())
            .await;

        // Peer manager receives a request to connect to the other peer.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(other_peer_id, other_addr.clone())
            .await;

        // Send request to connect to other peer at old address. ConnectivityManager should not
        // dial, since we are already connected at the new address.
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, update)
            .await;
        mock.trigger_connectivity_check().await;
        assert_eq!(0, mock.get_dial_queue_size().await);

        // Sending new address of other peer
        let (other_peer_new, other_addr_new) =
            update_peer_with_address(other_peer, "/ip4/127.0.1.1/tcp/8080");
        let update = hashmap! {other_peer_id => other_peer_new};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, update)
            .await;
        mock.trigger_connectivity_check().await;
        assert_eq!(1, mock.get_connected_size().await);

        // We expect the peer which changed its address to also disconnect. (even if the address doesn't match storage)
        mock.send_lost_peer_await_delivery(other_peer_id, other_addr_new.clone())
            .await;
        assert_eq!(0, mock.get_connected_size().await);

        // We should receive dial request to other peer at new address
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(other_peer_id, other_addr_new)
            .await;
    };
    block_on(future::join(conn_mgr.start(), test));
}

#[test]
fn lost_connection() {
    let (other_peer_id, other_peer, _, other_addr) = test_peer(0);
    let (mut mock, conn_mgr) = TestHarness::new(HashMap::new());

    let test = async move {
        // Sending address of other peer
        let update = hashmap! {other_peer_id => other_peer};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, update)
            .await;

        // Peer manager receives a request to connect to the other peer.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(other_peer_id, other_addr.clone())
            .await;

        // Sending LostPeer event to signal connection loss
        mock.send_lost_peer_await_delivery(other_peer_id, other_addr.clone())
            .await;

        // Peer manager receives a request to connect to the other peer after loss of
        // connection.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(other_peer_id, other_addr.clone())
            .await;
    };
    block_on(future::join(conn_mgr.start(), test));
}

#[test]
fn disconnect() {
    let (other_peer_id, other_peer, _, other_addr) = test_peer(0);
    let (mut mock, conn_mgr) = TestHarness::new(HashMap::new());

    let test = async move {
        // Sending pubkey & address of other peer
        let peers = hashmap! {other_peer_id => other_peer.clone()};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, peers)
            .await;

        // Waiting to receive dial request
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(other_peer_id, other_addr.clone())
            .await;

        // Sending request to make other peer ineligible (by dropping the key)
        let mut peer = other_peer;
        peer.keys = HashSet::new();
        peer.addresses = vec![network_address(DEFAULT_BASE_ADDR)];

        let peers = hashmap! {other_peer_id => peer};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, peers)
            .await;

        // Peer is now ineligible, we should disconnect from them
        mock.trigger_connectivity_check().await;
        mock.expect_disconnect_success(other_peer_id, other_addr)
            .await;
    };
    block_on(future::join(conn_mgr.start(), test));
}

// Tests that connectivity manager retries dials and disconnects on failure.
#[test]
fn retry_on_failure() {
    let (other_peer_id, peer, _, other_addr) = test_peer(0);
    let (mut mock, conn_mgr) = TestHarness::new(HashMap::new());

    let test = async move {
        // Sending pubkey set and addr of other peer
        let peers = hashmap! {other_peer_id => peer.clone()};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, peers)
            .await;

        // First dial attempt fails
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_fail(other_peer_id, other_addr.clone())
            .await;

        // We should retry after the failed attempt; this time, it succeeds.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(other_peer_id, other_addr.clone())
            .await;

        // Sending request to make other peer ineligible (by removing key)
        let mut peer = peer;
        peer.keys = HashSet::new();
        peer.addresses = vec![network_address(DEFAULT_BASE_ADDR)];
        let peers = hashmap! {other_peer_id => peer};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, peers)
            .await;

        // Peer manager receives a request to disconnect from the other peer, which fails.
        mock.trigger_connectivity_check().await;
        mock.expect_disconnect_fail(other_peer_id, other_addr.clone())
            .await;

        // Peer manager receives another request to disconnect from the other peer, which now
        // succeeds.
        mock.trigger_connectivity_check().await;
        mock.expect_disconnect_success(other_peer_id, other_addr.clone())
            .await;
    };
    block_on(future::join(conn_mgr.start(), test));
}

// Tests that if we dial an already connected peer or disconnect from an already disconnected
// peer, connectivity manager does not send any additional dial or disconnect requests.
#[test]
fn no_op_requests() {
    let (other_peer_id, peer, _, other_addr) = test_peer(0);
    let (mut mock, conn_mgr) = TestHarness::new(HashMap::new());

    let test = async move {
        // Sending pubkey set and addr of other peer
        let peers = hashmap! {other_peer_id => peer.clone()};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, peers)
            .await;

        // Peer manager receives a request to connect to the other peer.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_fail(other_peer_id, other_addr.clone())
            .await;

        // Send a delayed NewPeer notification.
        mock.send_new_peer_await_delivery(other_peer_id, other_peer_id, other_addr.clone())
            .await;
        mock.trigger_connectivity_check().await;

        // Send request to make other peer ineligible.
        let mut peer = peer;
        peer.keys = HashSet::new();
        peer.addresses = vec![network_address(DEFAULT_BASE_ADDR)];
        let peers = hashmap! {other_peer_id => peer};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, peers)
            .await;

        // Peer manager receives a request to disconnect from the other peer, which fails.
        mock.trigger_connectivity_check().await;
        mock.expect_disconnect_fail(other_peer_id, other_addr.clone())
            .await;

        // Send delayed LostPeer notification for other peer.
        mock.send_lost_peer_await_delivery(other_peer_id, other_addr)
            .await;

        // Trigger connectivity check again. We don't expect connectivity manager to do
        // anything - if it does, the task should panic. That may not fail the test (right
        // now), but will be easily spotted by someone running the tests locally.
        mock.trigger_connectivity_check().await;
        assert_eq!(0, mock.get_connected_size().await);
        assert_eq!(0, mock.get_dial_queue_size().await);
    };
    block_on(future::join(conn_mgr.start(), test));
}

#[test]
fn backoff_on_failure() {
    let (mut mock, conn_mgr) = TestHarness::new(HashMap::new());

    let test = async move {
        let (peer_id_a, peer_a, _, peer_a_addr) = test_peer(1);
        let (peer_id_b, peer_b, _, peer_b_addr) = test_peer(2);

        // Sending pubkey set and addr of peers
        let peers = hashmap! {peer_id_a => peer_a, peer_id_b => peer_b};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, peers)
            .await;

        // Send NewPeer notification for peer_b.
        mock.send_new_peer_await_delivery(peer_id_b, peer_id_b, peer_b_addr)
            .await;

        // We fail 10 attempts. In production, an exponential backoff strategy is used.
        for _ in 0..10 {
            // Peer manager receives a request to connect to the seed peer.
            mock.trigger_connectivity_check().await;
            mock.trigger_pending_dials().await;
            mock.expect_one_dial_fail(peer_id_a, peer_a_addr.clone())
                .await;
        }

        // Finally, one dial request succeeds
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(peer_id_a, peer_a_addr).await;
    };
    block_on(future::join(conn_mgr.start(), test));
}

// Test that connectivity manager will still connect to a peer if it advertises
// multiple listen addresses and some of them don't work.
#[test]
fn multiple_addrs_basic() {
    let (other_peer_id, mut peer, pubkey, _) = test_peer(0);
    let (mut mock, conn_mgr) = TestHarness::new(HashMap::new());

    let test = async move {
        // For this test, the peer advertises multiple listen addresses. Assume
        // that the first addr fails to connect while the second addr succeeds.
        let other_addr_1 = network_address_with_pubkey("/ip4/127.0.0.1/tcp/9091", pubkey);
        let other_addr_2 = network_address_with_pubkey("/ip4/127.0.0.1/tcp/9092", pubkey);
        peer.addresses = vec![other_addr_1.clone(), other_addr_2.clone()];

        // Sending address of other peer
        let update = hashmap! {other_peer_id => peer};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, update)
            .await;

        // Assume that the first listen addr fails to connect.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_fail(other_peer_id, other_addr_1).await;

        // Since the last connection attempt failed for other_addr_1, we should
        // attempt the next available listener address. In this case, the call
        // succeeds and we should connect to the peer.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(other_peer_id, other_addr_2)
            .await;
    };
    block_on(future::join(conn_mgr.start(), test));
}

// Test that connectivity manager will work with multiple addresses even if we
// retry more times than there are addresses.
#[test]
fn multiple_addrs_wrapping() {
    let (other_peer_id, mut peer, pubkey, _) = test_peer(0);
    let (mut mock, conn_mgr) = TestHarness::new(HashMap::new());

    let test = async move {
        let other_addr_1 = network_address_with_pubkey("/ip4/127.0.0.1/tcp/9091", pubkey);
        let other_addr_2 = network_address_with_pubkey("/ip4/127.0.0.1/tcp/9092", pubkey);
        peer.addresses = vec![other_addr_1.clone(), other_addr_2.clone()];

        // Sending address of other peer
        let update = hashmap! {other_peer_id => peer};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, update)
            .await;

        // Assume that the first listen addr fails to connect.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_fail(other_peer_id, other_addr_1.clone())
            .await;

        // The second attempt also fails.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_fail(other_peer_id, other_addr_2).await;

        // Our next attempt should wrap around to the first address.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(other_peer_id, other_addr_1)
            .await;
    };
    block_on(future::join(conn_mgr.start(), test));
}

// Test that connectivity manager will still work when dialing a peer with
// multiple listen addrs and then that peer advertises a smaller number of addrs.
#[test]
fn multiple_addrs_shrinking() {
    let (other_peer_id, mut peer, pubkey, _) = test_peer(0);
    let (mut mock, conn_mgr) = TestHarness::new(HashMap::new());

    let test = async move {
        let other_addr_1 = network_address_with_pubkey("/ip4/127.0.0.1/tcp/9091", pubkey);
        let other_addr_2 = network_address_with_pubkey("/ip4/127.0.0.1/tcp/9092", pubkey);
        let other_addr_3 = network_address_with_pubkey("/ip4/127.0.0.1/tcp/9093", pubkey);
        peer.addresses = vec![other_addr_1.clone(), other_addr_2, other_addr_3];

        // Sending address of other peer
        let update = hashmap! {other_peer_id => peer.clone()};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, update)
            .await;

        // Assume that the first listen addr fails to connect.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_fail(other_peer_id, other_addr_1).await;

        let other_addr_4 = network_address_with_pubkey("/ip4/127.0.0.1/tcp/9094", pubkey);
        let other_addr_5 = network_address_with_pubkey("/ip4/127.0.0.1/tcp/9095", pubkey);
        peer.addresses = vec![other_addr_4.clone(), other_addr_5];

        // The peer issues a new, smaller set of listen addrs.
        let update = hashmap! {other_peer_id => peer};
        mock.send_update_discovered_peers(DiscoverySource::OnChainValidatorSet, update)
            .await;

        // After updating the addresses, we should dial the first new address,
        // other_addr_4 in this case.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(other_peer_id, other_addr_4)
            .await;
    };
    block_on(future::join(conn_mgr.start(), test));
}

#[test]
fn public_connection_limit() {
    let mut seeds = HashMap::new();

    for i in 0..=MAX_TEST_CONNECTIONS {
        let (peer_id, peer, _, _) = test_peer(i);
        seeds.insert(peer_id, peer);
    }

    let (mut mock, conn_mgr) = TestHarness::new(seeds);

    let test = async move {
        // Should receive MAX_TEST_CONNECTIONS for each of the seed peers
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_num_dials(MAX_TEST_CONNECTIONS).await;

        // Should be expected number of connnections
        assert_eq!(MAX_TEST_CONNECTIONS, mock.get_connected_size().await);

        // Should be no more pending dials, even after a connectivity check.
        mock.trigger_connectivity_check().await;
        assert_eq!(0, mock.get_dial_queue_size().await);
    };
    block_on(future::join(conn_mgr.start(), test));
}

#[test]
fn basic_update_discovered_peers() {
    let mut rng = StdRng::from_seed(TEST_SEED);
    let (mock, mut conn_mgr) = TestHarness::new(HashMap::new());
    let trusted_peers = mock.trusted_peers;

    // sample some example data
    let peer_id_a = peer_id(0);
    let peer_id_b = peer_id(1);
    let addr_a = network_address("/ip4/127.0.0.1/tcp/9090");
    let addr_b = network_address("/ip4/127.0.0.1/tcp/9091");
    let pubkey_1 = x25519::PrivateKey::generate(&mut rng).public_key();
    let pubkey_2 = x25519::PrivateKey::generate(&mut rng).public_key();

    let pubkeys_1 = hashset! {pubkey_1};
    let pubkeys_2 = hashset! {pubkey_2};
    let pubkeys_1_2 = hashset! {pubkey_1, pubkey_2};

    let peer_a1 = Peer::new(vec![addr_a.clone()], pubkeys_1.clone(), PeerRole::Validator);
    let peer_a2 = Peer::new(vec![addr_a.clone()], pubkeys_2, PeerRole::Validator);
    let peer_b1 = Peer::new(vec![addr_b], pubkeys_1, PeerRole::Validator);
    let peer_a_1_2 = Peer::new(vec![addr_a], pubkeys_1_2, PeerRole::Validator);

    let peers_empty = PeerSet::new();
    let peers_1 = hashmap! {peer_id_a => peer_a1};
    let peers_2 = hashmap! {peer_id_a => peer_a2, peer_id_b => peer_b1.clone()};
    let peers_1_2 = hashmap! {peer_id_a => peer_a_1_2, peer_id_b => peer_b1};

    // basic one peer one discovery source
    conn_mgr.handle_update_discovered_peers(DiscoverySource::OnChainValidatorSet, peers_1.clone());
    assert_eq!(*trusted_peers.read(), peers_1);

    // same update does nothing
    conn_mgr.handle_update_discovered_peers(DiscoverySource::OnChainValidatorSet, peers_1.clone());
    assert_eq!(*trusted_peers.read(), peers_1);

    // reset
    conn_mgr
        .handle_update_discovered_peers(DiscoverySource::OnChainValidatorSet, peers_empty.clone());
    assert_eq!(*trusted_peers.read(), peers_empty);

    // basic union across multiple sources
    conn_mgr.handle_update_discovered_peers(DiscoverySource::OnChainValidatorSet, peers_1.clone());
    assert_eq!(*trusted_peers.read(), peers_1);
    conn_mgr.handle_update_discovered_peers(DiscoverySource::Config, peers_2);
    assert_eq!(*trusted_peers.read(), peers_1_2);

    // does nothing even if another source has same set
    conn_mgr
        .handle_update_discovered_peers(DiscoverySource::OnChainValidatorSet, peers_1_2.clone());
    assert_eq!(*trusted_peers.read(), peers_1_2);
    conn_mgr.handle_update_discovered_peers(DiscoverySource::Config, peers_1_2.clone());
    assert_eq!(*trusted_peers.read(), peers_1_2);

    // since on-chain and config now contain the same sets, clearing one should do nothing.
    conn_mgr.handle_update_discovered_peers(DiscoverySource::Config, peers_empty.clone());
    assert_eq!(*trusted_peers.read(), peers_1_2);

    // reset
    conn_mgr
        .handle_update_discovered_peers(DiscoverySource::OnChainValidatorSet, peers_empty.clone());
    assert_eq!(*trusted_peers.read(), peers_empty);

    // empty update again does nothing
    conn_mgr.handle_update_discovered_peers(DiscoverySource::Config, peers_empty.clone());
    assert_eq!(*trusted_peers.read(), peers_empty);
}
