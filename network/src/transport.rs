// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::NetworkPublicKeys,
    noise::{stream::NoiseStream, HandshakeAuthMode, NoiseUpgrader},
    protocols::{
        identity::exchange_handshake,
        wire::handshake::v1::{HandshakeMsg, MessagingProtocolVersion, SupportedProtocols},
    },
};
use futures::{
    future::{Future, FutureExt},
    io::{AsyncRead, AsyncWrite},
    stream::{Stream, StreamExt, TryStreamExt},
};
use libra_config::{config::HANDSHAKE_VERSION, network_id::NetworkId};
use libra_crypto::x25519;
use libra_logger::prelude::*;
use libra_network_address::{parse_dns_tcp, parse_ip_tcp, parse_memory, NetworkAddress};
use libra_security_logger::{security_log, SecurityEvent};
use libra_types::PeerId;
use netcore::transport::{tcp, ConnectionOrigin, Transport};
use std::{
    collections::HashMap,
    convert::TryFrom,
    fmt::Debug,
    io,
    pin::Pin,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, RwLock,
    },
    time::Duration,
};
use tokio::time::timeout;

/// A timeout for the connection to open and complete all of the upgrade steps.
pub const TRANSPORT_TIMEOUT: Duration = Duration::from_secs(30);

/// Currently supported messaging protocol version.
/// TODO: Add ability to support more than one messaging protocol.
pub const SUPPORTED_MESSAGING_PROTOCOL: MessagingProtocolVersion = MessagingProtocolVersion::V1;

/// Global connection-id generator.
static CONNECTION_ID_GENERATOR: ConnectionIdGenerator = ConnectionIdGenerator::new();

/// tcp::Transport with Libra-specific configuration applied.
pub const LIBRA_TCP_TRANSPORT: tcp::TcpTransport = tcp::TcpTransport {
    // Use default options.
    recv_buffer_size: None,
    send_buffer_size: None,
    ttl: None,
    keepalive: None,
    // Use TCP_NODELAY for libra tcp connections.
    nodelay: Some(true),
};

/// A trait alias for "socket-like" things.
pub trait TSocket: AsyncRead + AsyncWrite + Send + Debug + Unpin + 'static {}

impl<T> TSocket for T where T: AsyncRead + AsyncWrite + Send + Debug + Unpin + 'static {}

/// Unique local identifier for a connection.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct ConnectionId(u32);

impl From<u32> for ConnectionId {
    fn from(i: u32) -> ConnectionId {
        ConnectionId(i)
    }
}

/// Generator of unique ConnectionId's.
struct ConnectionIdGenerator {
    ctr: AtomicU32,
}

impl ConnectionIdGenerator {
    const fn new() -> ConnectionIdGenerator {
        Self {
            ctr: AtomicU32::new(0),
        }
    }

    fn next(&self) -> ConnectionId {
        let next = self.ctr.fetch_add(1, Ordering::Relaxed);
        ConnectionId::from(next)
    }
}

/// Metadata associated with an established and fully upgraded connection.
#[derive(Clone, Debug)]
pub struct ConnectionMetadata {
    peer_id: PeerId,
    connection_id: ConnectionId,
    addr: NetworkAddress,
    origin: ConnectionOrigin,
    messaging_protocol: MessagingProtocolVersion,
    application_protocols: SupportedProtocols,
}

impl ConnectionMetadata {
    pub fn new(
        peer_id: PeerId,
        connection_id: ConnectionId,
        addr: NetworkAddress,
        origin: ConnectionOrigin,
        messaging_protocol: MessagingProtocolVersion,
        application_protocols: SupportedProtocols,
    ) -> ConnectionMetadata {
        ConnectionMetadata {
            peer_id,
            connection_id,
            addr,
            origin,
            messaging_protocol,
            application_protocols,
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }

    pub fn addr(&self) -> &NetworkAddress {
        &self.addr
    }

    pub fn origin(&self) -> ConnectionOrigin {
        self.origin
    }
}

/// The `Connection` struct consists of connection metadata and the actual socket for
/// communication.
#[derive(Debug)]
pub struct Connection<TSocket> {
    pub socket: TSocket,
    pub metadata: ConnectionMetadata,
}

/// Convert a remote pubkey into a network `PeerId`.
fn identity_pubkey_to_peer_id(
    maybe_trusted_peers: Option<&Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>>,
    remote_pubkey: &x25519::PublicKey,
) -> io::Result<PeerId> {
    if let Some(trusted_peers) = maybe_trusted_peers {
        // if mutual authentication, try to find peer with this identity pubkey.

        let trusted_peers = trusted_peers.read().unwrap();
        let maybe_peerid_with_pubkey = trusted_peers
            .iter()
            .find(|(_peer_id, public_keys)| &public_keys.identity_public_key == remote_pubkey)
            .map(|(peer_id, _)| *peer_id);
        maybe_peerid_with_pubkey
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Not a trusted peer"))
            .map_err(|err| {
                security_log(SecurityEvent::InvalidNetworkPeer)
                    .error("UntrustedPeer")
                    .data(&trusted_peers)
                    .data(&remote_pubkey)
                    .log();
                err
            })
    } else {
        // else, only client authenticating the server. Generate PeerId from x25519::PublicKey.
        // Note: This is inconsistent with current types because AccountAddress is derived
        // from consensus key which is of type Ed25519PublicKey. Since AccountAddress does
        // not mean anything in a setting without remote authentication, we use the network
        // public key to generate a peer_id for the peer.
        // See this issue for potential improvements: https://github.com/libra/libra/issues/3960
        let mut array = [0u8; PeerId::LENGTH];
        let pubkey_slice = remote_pubkey.as_slice();
        // keep only the last 16 bytes
        array.copy_from_slice(&pubkey_slice[x25519::PUBLIC_KEY_SIZE - PeerId::LENGTH..]);
        Ok(PeerId::new(array))
    }
}

/// Exchange HandshakeMsg's to try negotiating a set of common supported protocols.
pub async fn perform_handshake<T: TSocket>(
    peer_id: PeerId,
    mut socket: T,
    addr: NetworkAddress,
    origin: ConnectionOrigin,
    own_handshake: &HandshakeMsg,
) -> io::Result<Connection<T>> {
    let handshake_other = exchange_handshake(&own_handshake, &mut socket).await?;
    if own_handshake.network_id != handshake_other.network_id {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "network_ids don't match own: {:?} received: {:?}",
                own_handshake.network_id, handshake_other.network_id
            ),
        ));
    }

    let intersecting_protocols = own_handshake.find_common_protocols(&handshake_other);
    match intersecting_protocols {
        None => {
            info!("No matching protocols found for connection with peer: {:?}. Handshake received: {:?}",
                  peer_id.short_str(), handshake_other);
            Err(io::Error::new(
                io::ErrorKind::Other,
                "no matching messaging protocol",
            ))
        }
        Some((messaging_protocol, application_protocols)) => Ok(Connection {
            socket,
            metadata: ConnectionMetadata::new(
                peer_id,
                CONNECTION_ID_GENERATOR.next(),
                addr,
                origin,
                messaging_protocol,
                application_protocols,
            ),
        }),
    }
}

/// Convenience function for adding a timeout to a Future that returns an `io::Result`.
async fn timeout_io<F, T>(duration: Duration, fut: F) -> io::Result<T>
where
    F: Future<Output = io::Result<T>>,
{
    let res = timeout(duration, fut).await;
    match res {
        Ok(out) => out,
        Err(err) => Err(io::Error::new(io::ErrorKind::TimedOut, err)),
    }
}

/// Common context for performing both inbound and outbound connection upgrades.
struct UpgradeContext {
    noise: NoiseUpgrader,
    trusted_peers: Option<Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>>,
    handshake_version: u8,
    own_handshake: HandshakeMsg,
}

/// Upgrade an inbound connection. This means we run a Noise IK handshake for
/// authentication and then negotiate common supported protocols. If
/// `ctxt.trusted_peers` is `Some(_)`, then we will only allow connections from
/// peers with a pubkey in this set. Otherwise, we will allow inbound connections
/// from any pubkey.
async fn upgrade_inbound<T: TSocket>(
    ctxt: Arc<UpgradeContext>,
    fut_socket: impl Future<Output = io::Result<T>>,
    addr: NetworkAddress,
) -> io::Result<Connection<NoiseStream<T>>> {
    let origin = ConnectionOrigin::Inbound;
    let socket = fut_socket.await?;

    // try authenticating via noise handshake
    let socket = ctxt.noise.upgrade_inbound(socket).await?;
    let remote_pubkey = socket.get_remote_static();

    let peer_id = identity_pubkey_to_peer_id(ctxt.trusted_peers.as_ref(), &remote_pubkey)?;
    let addr = addr.append_prod_protos(remote_pubkey, HANDSHAKE_VERSION);

    // try to negotiate common libranet version and supported application protocols
    perform_handshake(peer_id, socket, addr, origin, &ctxt.own_handshake).await
}

/// Upgrade an inbound connection. This means we run a Noise IK handshake for
/// authentication and then negotiate common supported protocols.
async fn upgrade_outbound<T: TSocket>(
    ctxt: Arc<UpgradeContext>,
    fut_socket: impl Future<Output = io::Result<T>>,
    addr: NetworkAddress,
    remote_pubkey: x25519::PublicKey,
) -> io::Result<Connection<NoiseStream<T>>> {
    let origin = ConnectionOrigin::Outbound;
    let socket = fut_socket.await?;

    // we can check early if this is even a trusted peer
    let peer_id = identity_pubkey_to_peer_id(ctxt.trusted_peers.as_ref(), &remote_pubkey)?;

    // try authenticating via noise handshake
    let socket = ctxt.noise.upgrade_outbound(socket, remote_pubkey).await?;

    // sanity check: Noise IK should always guarantee this is true
    debug_assert_eq!(remote_pubkey, socket.get_remote_static());

    // try to negotiate common libranet version and supported application protocols
    perform_handshake(peer_id, socket, addr, origin, &ctxt.own_handshake).await
}

/// The common LibraNet Transport.
///
/// The base transport layer is pluggable, so long as it provides a reliable,
/// ordered, connection-oriented, byte-stream abstraction (e.g., TCP). We currently
/// use either `MemoryTransport` or `TcpTransport` as this base layer.
///
/// Inbound and outbound connections are first established with the `base_transport`
/// and then negotiate a secure, authenticated transport layer (currently Noise
/// protocol). Finally, we negotiate common supported application protocols with
/// the `Handshake` protocol.
// TODO(philiphayes): rework Transport trait, possibly include Upgrade trait.
// ideas in this PR thread: https://github.com/libra/libra/pull/3478#issuecomment-617385633
pub struct LibraNetTransport<TTransport> {
    base_transport: TTransport,
    ctxt: Arc<UpgradeContext>,
    identity_pubkey: x25519::PublicKey,
}

impl<TTransport> LibraNetTransport<TTransport>
where
    TTransport: Transport<Error = io::Error>,
    TTransport::Output: TSocket,
    TTransport::Outbound: Send + 'static,
    TTransport::Inbound: Send + 'static,
    TTransport::Listener: Send + 'static,
{
    pub fn new(
        base_transport: TTransport,
        identity_key: x25519::PrivateKey,
        trusted_peers: Option<Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>>,
        handshake_version: u8,
        network_id: NetworkId,
        application_protocols: SupportedProtocols,
    ) -> Self {
        let mut own_handshake = HandshakeMsg::new(network_id);
        own_handshake.add(SUPPORTED_MESSAGING_PROTOCOL, application_protocols);
        let identity_pubkey = identity_key.public_key();

        let auth_mode = match trusted_peers.as_ref() {
            Some(trusted_peers) => HandshakeAuthMode::mutual(trusted_peers.clone()),
            None => HandshakeAuthMode::ServerOnly,
        };

        Self {
            ctxt: Arc::new(UpgradeContext {
                noise: NoiseUpgrader::new(identity_key, auth_mode),
                trusted_peers,
                handshake_version,
                own_handshake,
            }),
            base_transport,
            identity_pubkey,
        }
    }

    fn parse_dial_addr(
        addr: &NetworkAddress,
    ) -> io::Result<(NetworkAddress, x25519::PublicKey, u8)> {
        use libra_network_address::Protocol::*;

        let protos = addr.as_slice();

        // parse out the base transport protocol(s), which we will just ignore
        // and leave for the base_transport to actually parse and dial.
        // TODO(philiphayes): protos[..X] is kinda hacky. `Transport` trait
        // should handle this.
        let (base_transport_protos, base_transport_suffix) = parse_ip_tcp(protos)
            .map(|x| (&protos[..2], x.1))
            .or_else(|| parse_dns_tcp(protos).map(|x| (&protos[..2], x.1)))
            .or_else(|| parse_memory(protos).map(|x| (&protos[..1], x.1)))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Unexpected dialing network address: '{}', expected: \
                         memory, ip+tcp, or dns+tcp",
                        addr
                    ),
                )
            })?;

        // parse out the libranet protocols (noise ik and handshake)
        match base_transport_suffix {
            [NoiseIK(pubkey), Handshake(version)] => {
                let base_addr = NetworkAddress::try_from(base_transport_protos.to_vec())
                    .expect("base_transport_protos is always non-empty");
                Ok((base_addr, *pubkey, *version))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Unexpected dialing network address: '{}', expected: \
                     '/../ln-noise-ik/<pubkey>/ln-handshake/<version>'",
                    addr
                ),
            )),
        }
    }

    /// Dial a peer at `addr`. If the `addr` is not supported or formatted correctly,
    /// return `Err`. Otherwise, return a `Future` that resolves to `Err` if there
    /// was some issue dialing the peer and `Ok` with a fully upgraded connection
    /// to that peer if our dial was successful.
    ///
    /// ### Dialing `NetworkAddress` format
    ///
    /// We parse the dial address like:
    ///
    /// `/<base_transport>` + `/ln-noise-ik/<pubkey>/ln-handshake/<version>`
    ///
    /// If the base transport is `MemoryTransport`, then `/<base_transport>` is:
    ///
    /// `/memory/<port>`
    ///
    /// If the base transport is `TcpTransport`, then `/<base_transport>` is:
    ///
    /// `/ip4/<ipaddr>/tcp/<port>` or
    /// `/ip6/<ipaddr>/tcp/<port>` or
    /// `/dns/<ipaddr>/tcp/<port>` or
    /// `/dns4/<ipaddr>/tcp/<port>` or
    /// `/dns6/<ipaddr>/tcp/<port>`
    pub fn dial(
        &self,
        addr: NetworkAddress,
    ) -> io::Result<
        impl Future<Output = io::Result<Connection<NoiseStream<TTransport::Output>>>> + Send + 'static,
    > {
        // parse libranet protocols
        // TODO(philiphayes): `Transport` trait should include parsing in `dial`?
        let (base_addr, pubkey, handshake_version) = Self::parse_dial_addr(&addr)?;

        // Check that the parsed handshake version from the dial addr is supported.
        if self.ctxt.handshake_version != handshake_version {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Attempting to dial remote with unsupported handshake version: {}, expected: {}",
                    handshake_version, self.ctxt.handshake_version,
                ),
            ));
        }

        // try to connect socket
        let fut_socket = self.base_transport.dial(base_addr)?;

        // outbound dial upgrade task
        let upgrade_fut = upgrade_outbound(self.ctxt.clone(), fut_socket, addr, pubkey);
        let upgrade_fut = timeout_io(TRANSPORT_TIMEOUT, upgrade_fut);
        Ok(upgrade_fut)
    }

    /// Listen on address `addr`. If the `addr` is not supported or formatted correctly,
    /// return `Err`. Otherwise, return a `Stream` of fully upgraded inbound connections
    /// and the dialer's observed network address.
    ///
    /// ### Listening `NetworkAddress` format
    ///
    /// When listening, we only expect the base transport format. For example,
    /// if the base transport is `MemoryTransport`, then we expect:
    ///
    /// `/memory/<port>`
    ///
    /// If the base transport is `TcpTransport`, then we expect:
    ///
    /// `/ip4/<ipaddr>/tcp/<port>` or
    /// `/ip6/<ipaddr>/tcp/<port>`
    pub fn listen_on(
        &self,
        addr: NetworkAddress,
    ) -> io::Result<(
        impl Stream<
                Item = io::Result<(
                    impl Future<Output = io::Result<Connection<NoiseStream<TTransport::Output>>>>
                        + Send
                        + 'static,
                    NetworkAddress,
                )>,
            > + Send
            + 'static,
        NetworkAddress,
    )> {
        // listen on base transport. for example, this could be a tcp socket or
        // in-memory socket
        //
        // note: base transport should only accept its specific protocols
        // (e.g., `/memory/<port>` with no trailers), so we don't need to do any
        // parsing here.
        let (listener, listen_addr) = self.base_transport.listen_on(addr)?;
        let listen_addr =
            listen_addr.append_prod_protos(self.identity_pubkey, self.ctxt.handshake_version);

        // need to move a ctxt into stream task
        let ctxt = self.ctxt.clone();

        // stream of inbound upgrade tasks
        let inbounds = listener.map_ok(move |(fut_socket, addr)| {
            // inbound upgrade task
            let fut_upgrade = upgrade_inbound(ctxt.clone(), fut_socket, addr.clone());
            let fut_upgrade = timeout_io(TRANSPORT_TIMEOUT, fut_upgrade);
            (fut_upgrade, addr)
        });

        Ok((inbounds, listen_addr))
    }
}

// If using `LibraNetTransport` as a `Transport` trait, then all upgrade futures
// and listening streams must be boxed, since `upgrade_inbound` and `upgrade_outbound`
// are async fns (and therefore unnamed types).
//
// TODO(philiphayes): We can change these `Pin<Box<dyn Future<..>>> to `impl Future<..>`
// when/if this rust feature is stabilized: https://github.com/rust-lang/rust/issues/63063

impl<TTransport: Transport> Transport for LibraNetTransport<TTransport>
where
    TTransport: Transport<Error = io::Error> + Send + 'static,
    TTransport::Output: TSocket,
    TTransport::Outbound: Send + 'static,
    TTransport::Inbound: Send + 'static,
    TTransport::Listener: Send + 'static,
{
    type Output = Connection<NoiseStream<TTransport::Output>>;
    type Error = io::Error;
    type Inbound = Pin<Box<dyn Future<Output = io::Result<Self::Output>> + Send + 'static>>;
    type Outbound = Pin<Box<dyn Future<Output = io::Result<Self::Output>> + Send + 'static>>;
    type Listener =
        Pin<Box<dyn Stream<Item = io::Result<(Self::Inbound, NetworkAddress)>> + Send + 'static>>;

    fn dial(&self, addr: NetworkAddress) -> io::Result<Self::Outbound> {
        self.dial(addr).map(|upgrade_fut| upgrade_fut.boxed())
    }

    fn listen_on(&self, addr: NetworkAddress) -> io::Result<(Self::Listener, NetworkAddress)> {
        let (listener, listen_addr) = self.listen_on(addr)?;
        let listener = listener
            .map_ok(|(upgrade_fut, addr)| (upgrade_fut.boxed(), addr))
            .boxed();
        Ok((listener, listen_addr))
    }
}

// TODO(philiphayes): move tests into separate file

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        common::NetworkPublicKeys,
        protocols::wire::handshake::v1::{ProtocolId, SupportedProtocols},
    };
    use bytes::{Bytes, BytesMut};
    use futures::{executor::block_on, future, io::AsyncWriteExt};
    use libra_crypto::{test_utils::TEST_SEED, traits::Uniform};
    use libra_network_address::Protocol::*;
    use memsocket::MemorySocket;
    use netcore::{
        framing::{read_u16frame, write_u16frame},
        transport::memory,
    };
    use rand::{rngs::StdRng, SeedableRng};
    use tokio::runtime::Runtime;

    fn build_trusted_peers(
        id1: PeerId,
        key1: &x25519::PrivateKey,
        id2: PeerId,
        key2: &x25519::PrivateKey,
    ) -> Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>> {
        let pubkeys1 = NetworkPublicKeys {
            identity_public_key: key1.public_key(),
        };
        let pubkeys2 = NetworkPublicKeys {
            identity_public_key: key2.public_key(),
        };
        Arc::new(RwLock::new(
            vec![(id1, pubkeys1), (id2, pubkeys2)].into_iter().collect(),
        ))
    }

    enum Auth {
        Mutual,
        ServerOnly,
    }

    fn setup<TTransport>(
        base_transport: TTransport,
        auth: Auth,
    ) -> (
        Runtime,
        (PeerId, LibraNetTransport<TTransport>),
        (PeerId, LibraNetTransport<TTransport>),
        Option<Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>>,
        SupportedProtocols,
    )
    where
        TTransport: Transport<Error = io::Error> + Clone,
        TTransport::Output: TSocket,
        TTransport::Outbound: Send + 'static,
        TTransport::Inbound: Send + 'static,
        TTransport::Listener: Send + 'static,
    {
        let rt = Runtime::new().unwrap();

        let mut rng = StdRng::from_seed(TEST_SEED);
        let listener_key = x25519::PrivateKey::generate(&mut rng);
        let dialer_key = x25519::PrivateKey::generate(&mut rng);

        let (listener_peer_id, dialer_peer_id, trusted_peers) = match auth {
            Auth::Mutual => {
                let listener_peer_id = PeerId::random();
                let dialer_peer_id = PeerId::random();

                let trusted_peers = build_trusted_peers(
                    dialer_peer_id,
                    &dialer_key,
                    listener_peer_id,
                    &listener_key,
                );

                (listener_peer_id, dialer_peer_id, Some(trusted_peers))
            }
            Auth::ServerOnly => {
                let listener_peer_id =
                    identity_pubkey_to_peer_id(None, &listener_key.public_key()).unwrap();
                let dialer_peer_id =
                    identity_pubkey_to_peer_id(None, &dialer_key.public_key()).unwrap();

                (listener_peer_id, dialer_peer_id, None)
            }
        };

        let supported_protocols = SupportedProtocols::from(
            [ProtocolId::ConsensusRpc, ProtocolId::DiscoveryDirectSend].iter(),
        );

        let listener_transport = LibraNetTransport::new(
            base_transport.clone(),
            listener_key,
            trusted_peers.clone(),
            HANDSHAKE_VERSION,
            NetworkId::Validator,
            supported_protocols.clone(),
        );

        let dialer_transport = LibraNetTransport::new(
            base_transport,
            dialer_key,
            trusted_peers.clone(),
            HANDSHAKE_VERSION,
            NetworkId::Validator,
            supported_protocols.clone(),
        );

        (
            rt,
            (listener_peer_id, listener_transport),
            (dialer_peer_id, dialer_transport),
            trusted_peers,
            supported_protocols,
        )
    }

    async fn write_read_msg(socket: &mut impl TSocket, msg: &[u8]) -> Bytes {
        write_u16frame(socket, msg).await.unwrap();
        socket.flush().await.unwrap();

        let mut buf = BytesMut::new();
        read_u16frame(socket, &mut buf).await.unwrap();
        buf.freeze()
    }

    /// Check that the network address matches the format
    /// `"/memory/<port>/ln-noise-ik/<pubkey>/ln-handshake/<version>"`
    fn expect_memory_noise_addr(addr: &NetworkAddress) {
        assert!(
            matches!(addr.as_slice(), [Memory(_), NoiseIK(_), Handshake(_)]),
            "addr: '{}'",
            addr
        );
    }

    /// Check that the network address matches the format
    /// `"/ip4/<ipaddr>/tcp/<port>/ln-noise-ik/<pubkey>/ln-handshake/<version>"`
    fn expect_ip4_tcp_noise_addr(addr: &NetworkAddress) {
        assert!(
            matches!(addr.as_slice(), [Ip4(_), Tcp(_), NoiseIK(_), Handshake(_)]),
            "addr: '{}'",
            addr
        );
    }

    fn test_transport_success<TTransport>(
        base_transport: TTransport,
        auth: Auth,
        listen_addr: &str,
        expect_formatted_addr: fn(&NetworkAddress),
    ) where
        TTransport: Transport<Error = io::Error> + Clone,
        TTransport::Output: TSocket,
        TTransport::Outbound: Send + 'static,
        TTransport::Inbound: Send + 'static,
        TTransport::Listener: Send + 'static,
    {
        let (
            mut rt,
            (listener_peer_id, listener_transport),
            (dialer_peer_id, dialer_transport),
            _trusted_peers,
            supported_protocols,
        ) = setup(base_transport, auth);

        let (mut inbounds, listener_addr) = rt.enter(|| {
            listener_transport
                .listen_on(listen_addr.parse().unwrap())
                .unwrap()
        });
        expect_formatted_addr(&listener_addr);
        let supported_protocols_clone = supported_protocols.clone();

        // we accept the dialer's inbound connection, check the connection metadata,
        // and verify that the upgraded socket actually works (sends and receives
        // bytes).
        let listener_task = async move {
            // accept one inbound connection from dialer
            let (inbound, _dialer_addr) = inbounds.next().await.unwrap().unwrap();
            let mut conn = inbound.await.unwrap();

            // check connection metadata
            assert_eq!(conn.metadata.peer_id, dialer_peer_id);
            expect_formatted_addr(&conn.metadata.addr);
            assert_eq!(conn.metadata.origin, ConnectionOrigin::Inbound);
            assert_eq!(
                conn.metadata.messaging_protocol,
                MessagingProtocolVersion::V1
            );
            assert_eq!(
                conn.metadata.application_protocols,
                supported_protocols_clone,
            );

            // test the socket works
            let msg = write_read_msg(&mut conn.socket, b"foobar").await;
            assert_eq!(&msg, b"barbaz".as_ref());
            conn.socket.close().await.unwrap();
        };

        // dial the listener, check the connection metadata, and verify that the
        // upgraded socket actually works (sends and receives bytes).
        let dialer_task = async move {
            // dial listener
            let mut conn = dialer_transport
                .dial(listener_addr.clone())
                .unwrap()
                .await
                .unwrap();

            // check connection metadata
            assert_eq!(conn.metadata.peer_id, listener_peer_id);
            assert_eq!(conn.metadata.addr, listener_addr);
            assert_eq!(conn.metadata.origin, ConnectionOrigin::Outbound);
            assert_eq!(
                conn.metadata.messaging_protocol,
                MessagingProtocolVersion::V1
            );
            assert_eq!(conn.metadata.application_protocols, supported_protocols);

            // test the socket works
            let msg = write_read_msg(&mut conn.socket, b"barbaz").await;
            assert_eq!(&msg, b"foobar".as_ref());
            conn.socket.close().await.unwrap();
        };

        rt.block_on(future::join(listener_task, dialer_task));
    }

    fn test_transport_rejects_unauthed_dialer<TTransport>(
        base_transport: TTransport,
        listen_addr: &str,
        expect_formatted_addr: fn(&NetworkAddress),
    ) where
        TTransport: Transport<Error = io::Error> + Clone,
        TTransport::Output: TSocket,
        TTransport::Outbound: Send + 'static,
        TTransport::Inbound: Send + 'static,
        TTransport::Listener: Send + 'static,
    {
        let (
            mut rt,
            (_listener_peer_id, listener_transport),
            (dialer_peer_id, dialer_transport),
            trusted_peers,
            _supported_protocols,
        ) = setup(base_transport, Auth::Mutual);

        // remove dialer from trusted_peers set
        trusted_peers
            .as_ref()
            .unwrap()
            .write()
            .unwrap()
            .remove(&dialer_peer_id)
            .unwrap();

        let (mut inbounds, listener_addr) = rt.enter(|| {
            listener_transport
                .listen_on(listen_addr.parse().unwrap())
                .unwrap()
        });
        expect_formatted_addr(&listener_addr);

        // we try to accept one inbound connection from the dialer. however, the
        // connection upgrade should fail because the dialer is not authenticated
        // (not in the trusted peers set).
        let listener_task = async move {
            let (inbound, _dialer_addr) = inbounds.next().await.unwrap().unwrap();
            inbound
                .await
                .expect_err("should fail because the dialer is not a trusted peer");
        };

        // we attempt to dial the listener. however, the connection upgrade should
        // fail because we are not authenticated.
        let dialer_task = async move {
            // dial listener
            let fut_upgrade = dialer_transport.dial(listener_addr.clone()).unwrap();
            fut_upgrade
                .await
                .expect_err("should fail because listener rejects our unauthed connection");
        };

        rt.block_on(future::join(listener_task, dialer_task));
    }

    ////////////////////////////////////////
    // LibraNetTransport<MemoryTransport> //
    ////////////////////////////////////////

    #[test]
    fn test_memory_transport_mutual_auth() {
        test_transport_success(
            memory::MemoryTransport,
            Auth::Mutual,
            "/memory/0",
            expect_memory_noise_addr,
        );
    }

    #[test]
    fn test_memory_transport_server_only_auth() {
        test_transport_success(
            memory::MemoryTransport,
            Auth::ServerOnly,
            "/memory/0",
            expect_memory_noise_addr,
        );
    }

    #[test]
    fn test_memory_transport_rejects_unauthed_dialer() {
        test_transport_rejects_unauthed_dialer(
            memory::MemoryTransport,
            "/memory/0",
            expect_memory_noise_addr,
        );
    }

    /////////////////////////////////////
    // LibraNetTransport<TcpTransport> //
    /////////////////////////////////////

    #[test]
    fn test_tcp_transport_mutual_auth() {
        test_transport_success(
            LIBRA_TCP_TRANSPORT.clone(),
            Auth::Mutual,
            "/ip4/127.0.0.1/tcp/0",
            expect_ip4_tcp_noise_addr,
        );
    }

    #[test]
    fn test_tcp_transport_server_only_auth() {
        test_transport_success(
            LIBRA_TCP_TRANSPORT.clone(),
            Auth::ServerOnly,
            "/ip4/127.0.0.1/tcp/0",
            expect_ip4_tcp_noise_addr,
        );
    }

    #[test]
    fn test_tcp_transport_rejects_unauthed_dialer() {
        test_transport_rejects_unauthed_dialer(
            LIBRA_TCP_TRANSPORT.clone(),
            "/ip4/127.0.0.1/tcp/0",
            expect_ip4_tcp_noise_addr,
        );
    }

    ///////////////////////
    // perform_handshake //
    ///////////////////////

    #[test]
    fn handshake_network_id_mismatch() {
        let (outbound, inbound) = MemorySocket::new_pair();

        let mut server_handshake = HandshakeMsg::new(NetworkId::Validator);
        // This is required to ensure that test doesn't get an error for a different reason
        server_handshake.add(
            MessagingProtocolVersion::V1,
            [ProtocolId::ConsensusDirectSend].iter().into(),
        );
        let mut client_handshake = server_handshake.clone();
        // Ensure client doesn't match networks
        client_handshake.network_id = NetworkId::Public;

        let server = async move {
            perform_handshake(
                PeerId::random(),
                inbound,
                NetworkAddress::mock(),
                ConnectionOrigin::Inbound,
                &server_handshake,
            )
            .await
            .unwrap_err()
        };

        let client = async move {
            perform_handshake(
                PeerId::random(),
                outbound,
                NetworkAddress::mock(),
                ConnectionOrigin::Outbound,
                &client_handshake,
            )
            .await
            .unwrap_err()
        };

        block_on(future::join(server, client));
    }
}
