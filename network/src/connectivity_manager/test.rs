// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    peer::DisconnectReason,
    peer_manager::{conn_notifs_channel, ConnectionRequest},
    transport::ConnectionMetadata,
};
use channel::{diem_channel, message_queues::QueueStyle};
use diem_config::config::{Peer, PeerRole, PeerSet};
use diem_crypto::{test_utils::TEST_SEED, x25519, Uniform};
use diem_logger::info;
use diem_network_address::NetworkAddress;
use diem_time_service::{MockTimeService, TimeService};
use futures::{executor::block_on, future, SinkExt};
use rand::rngs::StdRng;
use std::{io, str::FromStr};
use tokio_retry::strategy::FixedInterval;

const MAX_TEST_CONNECTIONS: usize = 3;
const CONNECTIVITY_CHECK_INTERVAL: Duration = Duration::from_secs(5);
const CONNECTION_DELAY: Duration = Duration::from_millis(100);
const MAX_CONNECTION_DELAY: Duration = Duration::from_secs(60);

// TODO(philiphayes): just use `CONNECTION_DELAY + MAX_CONNNECTION_DELAY_JITTER`
// when the const adds are stabilized, instead of this weird thing...
const MAX_DELAY_WITH_JITTER: Duration = Duration::from_millis(
    CONNECTION_DELAY.as_millis() as u64 + MAX_CONNECTION_DELAY_JITTER.as_millis() as u64,
);

fn gen_peer() -> (
    PeerId,
    x25519::PublicKey,
    HashSet<x25519::PublicKey>,
    NetworkAddress,
) {
    let peer_id = PeerId::random();
    let pubkey = x25519::PrivateKey::generate_for_testing().public_key();
    let pubkeys: HashSet<_> = [pubkey].iter().copied().collect();
    let addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();
    let addr = addr.append_prod_protos(pubkey, diem_config::config::HANDSHAKE_VERSION);
    (peer_id, pubkey, pubkeys, addr)
}

struct TestHarness {
    trusted_peers: Arc<RwLock<PeerSet>>,
    mock_time: MockTimeService,
    connection_reqs_rx: diem_channel::Receiver<PeerId, ConnectionRequest>,
    connection_notifs_tx: conn_notifs_channel::Sender,
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
}

impl TestHarness {
    fn new(
        eligible_peers: Vec<PeerId>,
        seed_addrs: HashMap<PeerId, Vec<NetworkAddress>>,
    ) -> (Self, ConnectivityManager<FixedInterval>) {
        ::diem_logger::Logger::init_for_testing();

        // Fills in the gaps for eligible peers. Eligible peers in tests aren't
        // always the seeds.
        let seeds: PeerSet = eligible_peers
            .into_iter()
            .map(|peer_id| {
                let addrs: Vec<NetworkAddress> = seed_addrs
                    .get(&peer_id)
                    .map_or_else(Vec::new, |addr| addr.clone());
                let mut pubkeys = addrs
                    .iter()
                    .filter_map(NetworkAddress::find_noise_proto)
                    .collect::<HashSet<_>>();
                if pubkeys.is_empty() {
                    pubkeys.insert(x25519::PrivateKey::generate_for_testing().public_key());
                }
                (peer_id, Peer::new(addrs, pubkeys, PeerRole::Validator))
            })
            .collect();

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
            &seeds,
            ConnectionRequestSender::new(connection_reqs_tx),
            connection_notifs_rx,
            conn_mgr_reqs_rx,
            CONNECTIVITY_CHECK_INTERVAL,
            FixedInterval::new(CONNECTION_DELAY),
            MAX_CONNECTION_DELAY,
            Some(MAX_TEST_CONNECTIONS),
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
        let mut metadata = ConnectionMetadata::mock(notif_peer_id);
        metadata.addr = address;
        let notif = peer_manager::ConnectionNotification::NewPeer(metadata, NetworkContext::mock());
        self.send_notification_await_delivery(peer_id, notif).await;
    }

    async fn send_lost_peer_await_delivery(
        &mut self,
        peer_id: PeerId,
        notif_peer_id: PeerId,
        address: NetworkAddress,
        reason: DisconnectReason,
    ) {
        info!(
            "Sending LostPeer notification for peer: {}",
            peer_id.short_str()
        );
        let mut metadata = ConnectionMetadata::mock(notif_peer_id);
        metadata.addr = address;
        let notif = peer_manager::ConnectionNotification::LostPeer(
            metadata,
            NetworkContext::mock(),
            reason,
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

    async fn expect_disconnect_request(
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
            self.send_lost_peer_await_delivery(
                peer_id,
                peer_id,
                address,
                DisconnectReason::Requested,
            )
            .await;
        }
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

    async fn send_update_addresses(
        &mut self,
        src: DiscoverySource,
        peer_addresses: HashMap<PeerId, Vec<NetworkAddress>>,
    ) {
        let peers = peer_addresses
            .into_iter()
            .map(|(peer_id, addresses)| (peer_id, Peer::from_addrs(PeerRole::Validator, addresses)))
            .collect();
        self.send_update_discovered_peers(src, peers).await;
    }
}

fn check_trusted_peers(
    trusted_peers: &Arc<RwLock<PeerSet>>,
    expected_keys: &HashMap<PeerId, HashSet<x25519::PublicKey>>,
    expected_role: PeerRole,
) {
    let keys: HashMap<_, _> = trusted_peers
        .read()
        .iter()
        .map(|(peer_id, auth_context)| (*peer_id, auth_context.keys.clone()))
        .collect();
    trusted_peers
        .read()
        .iter()
        .for_each(|(_, auth_context)| assert_eq!(expected_role, auth_context.role));
    assert_eq!(expected_keys, &keys);
}

#[test]
fn connect_to_seeds_on_startup() {
    let (seed_peer_id, _, _, seed_addr) = gen_peer();
    let seed_addrs: HashMap<_, _> = vec![(seed_peer_id, vec![seed_addr.clone()])]
        .into_iter()
        .collect();
    let eligible_peers = vec![seed_peer_id];
    let (mut mock, conn_mgr) = TestHarness::new(eligible_peers, seed_addrs);

    let test = async move {
        // Peer manager receives a request to connect to the other peer.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(seed_peer_id, seed_addr.clone())
            .await;

        // Sending an UpdateAddresses with the same seed address should not
        // trigger any dials.
        mock.send_update_addresses(
            DiscoverySource::OnChain,
            [(seed_peer_id, vec![seed_addr.clone()])]
                .iter()
                .cloned()
                .collect(),
        )
        .await;
        mock.trigger_connectivity_check().await;
        assert_eq!(0, mock.get_dial_queue_size().await);

        // Sending new address of seed peer
        let new_seed_addr = NetworkAddress::from_str("/ip4/127.0.1.1/tcp/8080").unwrap();
        mock.send_update_addresses(
            DiscoverySource::OnChain,
            [(seed_peer_id, vec![new_seed_addr.clone()])]
                .iter()
                .cloned()
                .collect(),
        )
        .await;

        // We expect the peer which changed its address to also disconnect.
        mock.send_lost_peer_await_delivery(
            seed_peer_id,
            seed_peer_id,
            seed_addr.clone(),
            DisconnectReason::ConnectionLost,
        )
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
    let other_peer_id = PeerId::random();
    let other_addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

    let eligible_peers = vec![other_peer_id];
    let seed_addrs = HashMap::new();
    let (mut mock, conn_mgr) = TestHarness::new(eligible_peers, seed_addrs);

    let test = async move {
        // Sending address of other peer
        mock.send_update_addresses(
            DiscoverySource::OnChain,
            [(other_peer_id, vec![other_addr.clone()])]
                .iter()
                .cloned()
                .collect(),
        )
        .await;

        // Peer manager receives a request to connect to the other peer.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(other_peer_id, other_addr.clone())
            .await;

        // Send request to connect to other peer at old address. ConnectivityManager should not
        // dial, since we are already connected at the new address.
        mock.send_update_addresses(
            DiscoverySource::OnChain,
            [(other_peer_id, vec![other_addr.clone()])]
                .iter()
                .cloned()
                .collect(),
        )
        .await;
        mock.trigger_connectivity_check().await;
        assert_eq!(0, mock.get_dial_queue_size().await);

        // Sending new address of other peer
        let other_addr_new = NetworkAddress::from_str("/ip4/127.0.1.1/tcp/8080").unwrap();
        mock.send_update_addresses(
            DiscoverySource::OnChain,
            [(other_peer_id, vec![other_addr_new.clone()])]
                .iter()
                .cloned()
                .collect(),
        )
        .await;
        mock.trigger_connectivity_check().await;
        assert_eq!(1, mock.get_connected_size().await);

        // We expect the peer which changed its address to also disconnect. (even if the address doesn't match storage)
        mock.send_lost_peer_await_delivery(
            other_peer_id,
            other_peer_id,
            other_addr_new.clone(),
            DisconnectReason::ConnectionLost,
        )
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
    let other_peer_id = PeerId::random();
    let other_addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

    let eligible_peers = vec![other_peer_id];
    let seed_addrs = HashMap::new();
    let (mut mock, conn_mgr) = TestHarness::new(eligible_peers, seed_addrs);

    let test = async move {
        // Sending address of other peer
        mock.send_update_addresses(
            DiscoverySource::OnChain,
            [(other_peer_id, vec![other_addr.clone()])]
                .iter()
                .cloned()
                .collect(),
        )
        .await;

        // Peer manager receives a request to connect to the other peer.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(other_peer_id, other_addr.clone())
            .await;

        // Sending LostPeer event to signal connection loss
        mock.send_lost_peer_await_delivery(
            other_peer_id,
            other_peer_id,
            other_addr.clone(),
            DisconnectReason::ConnectionLost,
        )
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
    let other_peer_id = PeerId::random();
    let other_pubkey = x25519::PrivateKey::generate_for_testing().public_key();
    let other_pubkeys: HashSet<_> = [other_pubkey].iter().copied().collect();
    let other_addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

    let eligible_peers = vec![];
    let seed_addrs = HashMap::new();
    let (mut mock, conn_mgr) = TestHarness::new(eligible_peers, seed_addrs);

    let test = async move {
        // Sending pubkey & address of other peer
        let mut peers = PeerSet::new();
        peers.insert(
            other_peer_id,
            Peer::new(vec![other_addr.clone()], other_pubkeys, PeerRole::Validator),
        );
        mock.send_update_discovered_peers(DiscoverySource::OnChain, peers)
            .await;

        // Waiting to receive dial request
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(other_peer_id, other_addr.clone())
            .await;

        // Sending request to make other peer ineligible
        let mut peers = PeerSet::new();
        peers.insert(
            other_peer_id,
            Peer::new(
                vec![other_addr.clone()],
                HashSet::new(),
                PeerRole::Validator,
            ),
        );
        mock.send_update_discovered_peers(DiscoverySource::OnChain, peers)
            .await;

        // Peer is now ineligible, we should disconnect from them
        mock.trigger_connectivity_check().await;
        mock.expect_disconnect_request(other_peer_id, other_addr, Ok(()))
            .await;
    };
    block_on(future::join(conn_mgr.start(), test));
}

// Tests that connectivity manager retries dials and disconnects on failure.
#[test]
fn retry_on_failure() {
    let other_peer_id = PeerId::random();
    let other_pubkey = x25519::PrivateKey::generate_for_testing().public_key();
    let other_pubkeys: HashSet<_> = [other_pubkey].iter().copied().collect();
    let other_addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

    let eligible_peers = vec![];
    let seed_addrs = HashMap::new();
    let (mut mock, conn_mgr) = TestHarness::new(eligible_peers, seed_addrs);

    let test = async move {
        // Sending pubkey set and addr of other peer
        let mut peers = PeerSet::new();
        peers.insert(
            other_peer_id,
            Peer::new(vec![other_addr.clone()], other_pubkeys, PeerRole::Validator),
        );
        mock.send_update_discovered_peers(DiscoverySource::OnChain, peers)
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

        // Sending request to make other peer ineligible
        let mut peers = PeerSet::new();
        peers.insert(
            other_peer_id,
            Peer::new(
                vec![other_addr.clone()],
                HashSet::new(),
                PeerRole::Validator,
            ),
        );
        mock.send_update_discovered_peers(DiscoverySource::OnChain, peers)
            .await;

        // Peer manager receives a request to disconnect from the other peer, which fails.
        mock.trigger_connectivity_check().await;
        mock.expect_disconnect_request(
            other_peer_id,
            other_addr.clone(),
            Err(PeerManagerError::IoError(io::Error::from(
                io::ErrorKind::Interrupted,
            ))),
        )
        .await;

        // Peer manager receives another request to disconnect from the other peer, which now
        // succeeds.
        mock.trigger_connectivity_check().await;
        mock.expect_disconnect_request(other_peer_id, other_addr.clone(), Ok(()))
            .await;
    };
    block_on(future::join(conn_mgr.start(), test));
}

// Tests that if we dial an already connected peer or disconnect from an already disconnected
// peer, connectivity manager does not send any additional dial or disconnect requests.
#[test]
fn no_op_requests() {
    let other_peer_id = PeerId::random();
    let other_pubkey = x25519::PrivateKey::generate_for_testing().public_key();
    let other_pubkeys: HashSet<_> = [other_pubkey].iter().copied().collect();
    let other_addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

    let eligible_peers = vec![];
    let seed_addrs = HashMap::new();
    let (mut mock, conn_mgr) = TestHarness::new(eligible_peers, seed_addrs);

    let test = async move {
        // Sending pubkey set and addr of other peer
        let mut peers = PeerSet::new();
        peers.insert(
            other_peer_id,
            Peer::new(vec![other_addr.clone()], other_pubkeys, PeerRole::Validator),
        );
        mock.send_update_discovered_peers(DiscoverySource::OnChain, peers)
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
        let mut peers = PeerSet::new();
        peers.insert(
            other_peer_id,
            Peer::new(
                vec![other_addr.clone()],
                HashSet::new(),
                PeerRole::Validator,
            ),
        );
        mock.send_update_discovered_peers(DiscoverySource::OnChain, peers)
            .await;

        // Peer manager receives a request to disconnect from the other peer, which fails.
        mock.trigger_connectivity_check().await;
        mock.expect_disconnect_request(
            other_peer_id,
            other_addr.clone(),
            Err(PeerManagerError::NotConnected(other_peer_id)),
        )
        .await;

        // Send delayed LostPeer notification for other peer.
        mock.send_lost_peer_await_delivery(
            other_peer_id,
            other_peer_id,
            other_addr,
            DisconnectReason::ConnectionLost,
        )
        .await;

        // Trigger connectivity check again. We don't expect connectivity manager to do
        // anything - if it does, the task should panic. That may not fail the test (right
        // now), but will be easily spotted by someone running the tests locally.
        mock.trigger_connectivity_check().await;
    };
    block_on(future::join(conn_mgr.start(), test));
}

#[test]
fn backoff_on_failure() {
    let eligible_peers = vec![];
    let seed_addrs = HashMap::new();
    let (mut mock, conn_mgr) = TestHarness::new(eligible_peers, seed_addrs);

    let test = async move {
        let (peer_a, _, peer_a_keys, peer_a_addr) = gen_peer();
        let (peer_b, _, peer_b_keys, peer_b_addr) = gen_peer();

        // Sending pubkey set and addr of peers
        let mut peers = PeerSet::new();
        peers.insert(
            peer_a,
            Peer::new(vec![peer_a_addr.clone()], peer_a_keys, PeerRole::Validator),
        );
        peers.insert(
            peer_b,
            Peer::new(vec![peer_b_addr.clone()], peer_b_keys, PeerRole::Validator),
        );
        mock.send_update_discovered_peers(DiscoverySource::OnChain, peers)
            .await;

        // Send NewPeer notification for peer_b.
        mock.send_new_peer_await_delivery(peer_b, peer_b, peer_b_addr)
            .await;

        // We fail 10 attempts. In production, an exponential backoff strategy is used.
        for _ in 0..10 {
            // Peer manager receives a request to connect to the seed peer.
            mock.trigger_connectivity_check().await;
            mock.trigger_pending_dials().await;
            mock.expect_one_dial_fail(peer_a, peer_a_addr.clone()).await;
        }

        // Finally, one dial request succeeds
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_success(peer_a, peer_a_addr).await;
    };
    block_on(future::join(conn_mgr.start(), test));
}

// Test that connectivity manager will still connect to a peer if it advertises
// multiple listen addresses and some of them don't work.
#[test]
fn multiple_addrs_basic() {
    let other_peer_id = PeerId::random();
    let eligible_peers = vec![other_peer_id];
    let seed_addrs = HashMap::new();
    let (mut mock, conn_mgr) = TestHarness::new(eligible_peers, seed_addrs);

    let test = async move {
        // For this test, the peer advertises multiple listen addresses. Assume
        // that the first addr fails to connect while the second addr succeeds.
        let other_addr_1 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9091").unwrap();
        let other_addr_2 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9092").unwrap();

        // Sending address of other peer
        mock.send_update_addresses(
            DiscoverySource::OnChain,
            [(
                other_peer_id,
                vec![other_addr_1.clone(), other_addr_2.clone()],
            )]
            .iter()
            .cloned()
            .collect(),
        )
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
    let other_peer_id = PeerId::random();
    let eligible_peers = vec![other_peer_id];
    let seed_addrs = HashMap::new();
    let (mut mock, conn_mgr) = TestHarness::new(eligible_peers, seed_addrs);

    let test = async move {
        let other_addr_1 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9091").unwrap();
        let other_addr_2 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9092").unwrap();

        // Sending address of other peer
        mock.send_update_addresses(
            DiscoverySource::OnChain,
            [(
                other_peer_id,
                vec![other_addr_1.clone(), other_addr_2.clone()],
            )]
            .iter()
            .cloned()
            .collect(),
        )
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
    let other_peer_id = PeerId::random();
    let eligible_peers = vec![other_peer_id];
    let seed_addrs = HashMap::new();
    let (mut mock, conn_mgr) = TestHarness::new(eligible_peers, seed_addrs);

    let test = async move {
        let other_addr_1 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9091").unwrap();
        let other_addr_2 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9092").unwrap();
        let other_addr_3 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9092").unwrap();

        // Sending addresses of other peer
        mock.send_update_addresses(
            DiscoverySource::OnChain,
            [(
                other_peer_id,
                vec![
                    other_addr_1.clone(),
                    other_addr_2.clone(),
                    other_addr_3.clone(),
                ],
            )]
            .iter()
            .cloned()
            .collect(),
        )
        .await;

        // Assume that the first listen addr fails to connect.
        mock.trigger_connectivity_check().await;
        mock.trigger_pending_dials().await;
        mock.expect_one_dial_fail(other_peer_id, other_addr_1).await;

        let other_addr_4 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9094").unwrap();
        let other_addr_5 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9095").unwrap();

        // The peer issues a new, smaller set of listen addrs.
        mock.send_update_addresses(
            DiscoverySource::OnChain,
            [(
                other_peer_id,
                vec![other_addr_4.clone(), other_addr_5.clone()],
            )]
            .iter()
            .cloned()
            .collect(),
        )
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
    let mut seed_addrs = HashMap::new();
    let mut eligible_peers = Vec::new();

    for _ in 0..=MAX_TEST_CONNECTIONS {
        let (peer_id, _, _, addr) = gen_peer();
        seed_addrs.insert(peer_id, vec![addr]);
        eligible_peers.push(peer_id);
    }

    let (mut mock, conn_mgr) = TestHarness::new(eligible_peers, seed_addrs);

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
    let seed_addrs = HashMap::new();
    let eligible_peers = Vec::new();
    let (mock, mut conn_mgr) = TestHarness::new(eligible_peers, seed_addrs);
    let trusted_peers = mock.trusted_peers;

    // sample some example data
    let peer_id_a = PeerId::random();
    let peer_id_b = PeerId::random();
    let addr_a = NetworkAddress::mock();
    let addr_b = NetworkAddress::mock();

    let pubkey_1 = x25519::PrivateKey::generate(&mut rng).public_key();
    let pubkey_2 = x25519::PrivateKey::generate(&mut rng).public_key();

    let pubkeys_1: HashSet<_> = vec![pubkey_1].into_iter().collect();
    let pubkeys_2: HashSet<_> = vec![pubkey_2].into_iter().collect();
    let pubkeys_1_2: HashSet<_, _> = vec![pubkey_1, pubkey_2].into_iter().collect();

    let peer_a1 = Peer::new(vec![addr_a.clone()], pubkeys_1.clone(), PeerRole::Validator);
    let peer_a2 = Peer::new(vec![addr_a.clone()], pubkeys_2, PeerRole::Validator);
    let peer_b1 = Peer::new(vec![addr_b], pubkeys_1.clone(), PeerRole::Validator);
    let peer_a_1_2 = Peer::new(vec![addr_a], pubkeys_1_2.clone(), PeerRole::Validator);

    let pubkeys_map_empty = HashMap::new();
    let pubkeys_map_1: HashMap<_, _> = vec![(peer_id_a, pubkeys_1.clone())].into_iter().collect();
    let pubkeys_map_1_2: HashMap<_, _> = vec![(peer_id_a, pubkeys_1_2), (peer_id_b, pubkeys_1)]
        .into_iter()
        .collect();

    let peers_1: PeerSet = vec![(peer_id_a, peer_a1)].into_iter().collect();
    let peers_2: PeerSet = vec![(peer_id_a, peer_a2), (peer_id_b, peer_b1.clone())]
        .into_iter()
        .collect();
    let peers_1_2: PeerSet = vec![(peer_id_a, peer_a_1_2), (peer_id_b, peer_b1)]
        .into_iter()
        .collect();

    // basic one peer one discovery source
    conn_mgr.handle_update_discovered_peers(DiscoverySource::OnChain, peers_1.clone());
    check_trusted_peers(&trusted_peers, &pubkeys_map_1, PeerRole::Validator);

    // same update does nothing
    conn_mgr.handle_update_discovered_peers(DiscoverySource::OnChain, peers_1.clone());
    check_trusted_peers(&trusted_peers, &pubkeys_map_1, PeerRole::Validator);

    // reset
    conn_mgr.handle_update_discovered_peers(DiscoverySource::OnChain, PeerSet::new());
    check_trusted_peers(&trusted_peers, &pubkeys_map_empty, PeerRole::Validator);

    // basic union across multiple sources
    conn_mgr.handle_update_discovered_peers(DiscoverySource::OnChain, peers_1);
    check_trusted_peers(&trusted_peers, &pubkeys_map_1, PeerRole::Validator);
    conn_mgr.handle_update_discovered_peers(DiscoverySource::Config, peers_2);
    check_trusted_peers(&trusted_peers, &pubkeys_map_1_2, PeerRole::Validator);

    // does nothing even if another source has same set
    conn_mgr.handle_update_discovered_peers(DiscoverySource::OnChain, peers_1_2.clone());
    check_trusted_peers(&trusted_peers, &pubkeys_map_1_2, PeerRole::Validator);
    conn_mgr.handle_update_discovered_peers(DiscoverySource::Config, peers_1_2);
    check_trusted_peers(&trusted_peers, &pubkeys_map_1_2, PeerRole::Validator);

    // since on-chain and config now contain the same sets, clearing one should do nothing.
    conn_mgr.handle_update_discovered_peers(DiscoverySource::Config, PeerSet::new());
    check_trusted_peers(&trusted_peers, &pubkeys_map_1_2, PeerRole::Validator);

    // reset
    conn_mgr.handle_update_discovered_peers(DiscoverySource::OnChain, PeerSet::new());
    check_trusted_peers(&trusted_peers, &pubkeys_map_empty, PeerRole::Validator);

    // empty update again does nothing
    conn_mgr.handle_update_discovered_peers(DiscoverySource::Config, PeerSet::new());
    check_trusted_peers(&trusted_peers, &pubkeys_map_empty, PeerRole::Validator);
}
