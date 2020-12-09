// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    logging::NetworkSchema,
    noise::{stream::NoiseStream, AntiReplayTimestamps, HandshakeAuthMode, NoiseUpgrader},
    protocols::{
        identity::exchange_handshake,
        wire::handshake::v1::{HandshakeMsg, MessagingProtocolVersion, SupportedProtocols},
    },
};
use diem_config::{
    config::HANDSHAKE_VERSION,
    network_id::{NetworkContext, NetworkId},
};
use diem_crypto::x25519;
use diem_infallible::RwLock;
use diem_logger::prelude::*;
use diem_network_address::{parse_dns_tcp, parse_ip_tcp, parse_memory, NetworkAddress};
use diem_types::{chain_id::ChainId, PeerId};
use futures::{
    future::{Future, FutureExt},
    io::{AsyncRead, AsyncWrite},
    stream::{Stream, StreamExt, TryStreamExt},
};
use netcore::transport::{proxy_protocol, tcp, ConnectionOrigin, Transport};
use serde::{export::Formatter, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    convert::TryFrom,
    fmt::Debug,
    io,
    pin::Pin,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time::timeout;

#[cfg(test)]
mod test;

/// A timeout for the connection to open and complete all of the upgrade steps.
pub const TRANSPORT_TIMEOUT: Duration = Duration::from_secs(30);

/// Currently supported messaging protocol version.
/// TODO: Add ability to support more than one messaging protocol.
pub const SUPPORTED_MESSAGING_PROTOCOL: MessagingProtocolVersion = MessagingProtocolVersion::V1;

/// Global connection-id generator.
static CONNECTION_ID_GENERATOR: ConnectionIdGenerator = ConnectionIdGenerator::new();

/// tcp::Transport with Diem-specific configuration applied.
pub const DIEM_TCP_TRANSPORT: tcp::TcpTransport = tcp::TcpTransport {
    // Use default options.
    recv_buffer_size: None,
    send_buffer_size: None,
    ttl: None,
    keepalive: None,
    // Use TCP_NODELAY for diem tcp connections.
    nodelay: Some(true),
};

/// A trait alias for "socket-like" things.
pub trait TSocket: AsyncRead + AsyncWrite + Send + Debug + Unpin + 'static {}

impl<T> TSocket for T where T: AsyncRead + AsyncWrite + Send + Debug + Unpin + 'static {}

/// Unique local identifier for a connection.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize)]
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
#[derive(Clone, PartialEq, Serialize)]
pub struct ConnectionMetadata {
    pub remote_peer_id: PeerId,
    pub connection_id: ConnectionId,
    pub addr: NetworkAddress,
    pub origin: ConnectionOrigin,
    pub messaging_protocol: MessagingProtocolVersion,
    pub application_protocols: SupportedProtocols,
}

impl ConnectionMetadata {
    pub fn new(
        remote_peer_id: PeerId,
        connection_id: ConnectionId,
        addr: NetworkAddress,
        origin: ConnectionOrigin,
        messaging_protocol: MessagingProtocolVersion,
        application_protocols: SupportedProtocols,
    ) -> ConnectionMetadata {
        ConnectionMetadata {
            remote_peer_id,
            connection_id,
            addr,
            origin,
            messaging_protocol,
            application_protocols,
        }
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn mock(remote_peer_id: PeerId) -> ConnectionMetadata {
        ConnectionMetadata {
            remote_peer_id,
            connection_id: ConnectionId::default(),
            addr: NetworkAddress::mock(),
            origin: ConnectionOrigin::Inbound,
            messaging_protocol: MessagingProtocolVersion::V1,
            application_protocols: [].iter().into(),
        }
    }
}

impl std::fmt::Debug for ConnectionMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for ConnectionMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{},{},{},{},{:?}]",
            self.remote_peer_id,
            self.addr,
            self.origin,
            self.messaging_protocol,
            self.application_protocols
        )
    }
}

/// The `Connection` struct consists of connection metadata and the actual socket for
/// communication.
#[derive(Debug)]
pub struct Connection<TSocket> {
    pub socket: TSocket,
    pub metadata: ConnectionMetadata,
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
    handshake_version: u8,
    supported_protocols: BTreeMap<MessagingProtocolVersion, SupportedProtocols>,
    chain_id: ChainId,
    network_id: NetworkId,
}

/// If we have proxy protocol enabled, then prepend the un-proxied address to the error.
fn add_pp_addr(proxy_protocol_enabled: bool, error: io::Error, addr: &NetworkAddress) -> io::Error {
    if proxy_protocol_enabled {
        io::Error::new(
            error.kind(),
            format!("proxied address: {}, error: {}", addr, error),
        )
    } else {
        error
    }
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
    proxy_protocol_enabled: bool,
) -> io::Result<Connection<NoiseStream<T>>> {
    let origin = ConnectionOrigin::Inbound;
    let mut socket = fut_socket.await?;

    // If we have proxy protocol enabled, process the event, otherwise skip it
    // TODO: This would make more sense to build this in at instantiation so we don't need to put the if statement here
    let addr = if proxy_protocol_enabled {
        proxy_protocol::read_header(&addr, &mut socket)
            .await
            .map_err(|err| {
                debug!(
                    network_address = addr,
                    error = %err,
                    "ProxyProtocol: Failed to read header: {}",
                    err
                );
                err
            })?
    } else {
        addr
    };

    // try authenticating via noise handshake
    let (mut socket, remote_peer_id) = ctxt.noise.upgrade_inbound(socket).await.map_err(|err| {
        if err.should_security_log() {
            sample!(
                SampleRate::Duration(Duration::from_secs(15)),
                error!(
                    SecurityEvent::NoiseHandshake,
                    NetworkSchema::new(&ctxt.noise.network_context)
                        .network_address(&addr)
                        .connection_origin(&origin),
                    error = %err,
                )
            );
        }
        let err = io::Error::new(io::ErrorKind::Other, err);
        add_pp_addr(proxy_protocol_enabled, err, &addr)
    })?;
    let remote_pubkey = socket.get_remote_static();
    let addr = addr.append_prod_protos(remote_pubkey, HANDSHAKE_VERSION);

    // exchange HandshakeMsg
    let handshake_msg = HandshakeMsg {
        supported_protocols: ctxt.supported_protocols.clone(),
        chain_id: ctxt.chain_id,
        network_id: ctxt.network_id.clone(),
    };
    let remote_handshake = exchange_handshake(&handshake_msg, &mut socket)
        .await
        .map_err(|err| add_pp_addr(proxy_protocol_enabled, err, &addr))?;

    // try to negotiate common diemnet version and supported application protocols
    let (messaging_protocol, application_protocols) = handshake_msg
        .perform_handshake(&remote_handshake)
        .map_err(|err| {
            let err = format!(
                "handshake negotiation with peer {} failed: {}",
                remote_peer_id.short_str(),
                err
            );
            add_pp_addr(
                proxy_protocol_enabled,
                io::Error::new(io::ErrorKind::Other, err),
                &addr,
            )
        })?;

    // return successful connection
    Ok(Connection {
        socket,
        metadata: ConnectionMetadata::new(
            remote_peer_id,
            CONNECTION_ID_GENERATOR.next(),
            addr,
            origin,
            messaging_protocol,
            application_protocols,
        ),
    })
}

/// Upgrade an inbound connection. This means we run a Noise IK handshake for
/// authentication and then negotiate common supported protocols.
async fn upgrade_outbound<T: TSocket>(
    ctxt: Arc<UpgradeContext>,
    fut_socket: impl Future<Output = io::Result<T>>,
    addr: NetworkAddress,
    remote_peer_id: PeerId,
    remote_pubkey: x25519::PublicKey,
) -> io::Result<Connection<NoiseStream<T>>> {
    let origin = ConnectionOrigin::Outbound;
    let socket = fut_socket.await?;

    // noise handshake
    let mut socket = ctxt
        .noise
        .upgrade_outbound(socket, remote_pubkey, AntiReplayTimestamps::now)
        .await
        .map_err(|err| {
            if err.should_security_log() {
                sample!(
                    SampleRate::Duration(Duration::from_secs(15)),
                    error!(
                        SecurityEvent::NoiseHandshake,
                        NetworkSchema::new(&ctxt.noise.network_context)
                            .network_address(&addr)
                            .connection_origin(&origin),
                        error = %err,
                    )
                );
            }
            io::Error::new(io::ErrorKind::Other, err)
        })?;

    // sanity check: Noise IK should always guarantee this is true
    debug_assert_eq!(remote_pubkey, socket.get_remote_static());

    // exchange HandshakeMsg
    let handshake_msg = HandshakeMsg {
        supported_protocols: ctxt.supported_protocols.clone(),
        chain_id: ctxt.chain_id,
        network_id: ctxt.network_id.clone(),
    };
    let remote_handshake = exchange_handshake(&handshake_msg, &mut socket).await?;

    // try to negotiate common diemnet version and supported application protocols
    let (messaging_protocol, application_protocols) = handshake_msg
        .perform_handshake(&remote_handshake)
        .map_err(|e| {
            let e = format!(
                "handshake negotiation with peer {} failed: {}",
                remote_peer_id, e
            );
            io::Error::new(io::ErrorKind::Other, e)
        })?;

    // return successful connection
    Ok(Connection {
        socket,
        metadata: ConnectionMetadata::new(
            remote_peer_id,
            CONNECTION_ID_GENERATOR.next(),
            addr,
            origin,
            messaging_protocol,
            application_protocols,
        ),
    })
}

/// The common DiemNet Transport.
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
// ideas in this PR thread: https://github.com/diem/diem/pull/3478#issuecomment-617385633
pub struct DiemNetTransport<TTransport> {
    base_transport: TTransport,
    ctxt: Arc<UpgradeContext>,
    identity_pubkey: x25519::PublicKey,
    enable_proxy_protocol: bool,
}

impl<TTransport> DiemNetTransport<TTransport>
where
    TTransport: Transport<Error = io::Error>,
    TTransport::Output: TSocket,
    TTransport::Outbound: Send + 'static,
    TTransport::Inbound: Send + 'static,
    TTransport::Listener: Send + 'static,
{
    pub fn new(
        base_transport: TTransport,
        network_context: Arc<NetworkContext>,
        identity_key: x25519::PrivateKey,
        trusted_peers: Option<Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>>>,
        handshake_version: u8,
        chain_id: ChainId,
        application_protocols: SupportedProtocols,
        enable_proxy_protocol: bool,
    ) -> Self {
        // build supported protocols
        let mut supported_protocols = BTreeMap::new();
        supported_protocols.insert(SUPPORTED_MESSAGING_PROTOCOL, application_protocols);

        // create upgrade context
        // TODO(mimoo): should we build this based on the networkid, and not on trusted peers
        let auth_mode = match trusted_peers.as_ref() {
            Some(trusted_peers) => HandshakeAuthMode::mutual(trusted_peers.clone()),
            None => HandshakeAuthMode::ServerOnly,
        };
        let identity_pubkey = identity_key.public_key();
        let network_id = network_context.network_id().clone();

        let upgrade_context = UpgradeContext {
            noise: NoiseUpgrader::new(network_context, identity_key, auth_mode),
            handshake_version,
            supported_protocols,
            chain_id,
            network_id,
        };

        Self {
            ctxt: Arc::new(upgrade_context),
            base_transport,
            identity_pubkey,
            enable_proxy_protocol,
        }
    }

    fn parse_dial_addr(
        addr: &NetworkAddress,
    ) -> io::Result<(NetworkAddress, x25519::PublicKey, u8)> {
        use diem_network_address::Protocol::*;

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

        // parse out the diemnet protocols (noise ik and handshake)
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
        peer_id: PeerId,
        addr: NetworkAddress,
    ) -> io::Result<
        impl Future<Output = io::Result<Connection<NoiseStream<TTransport::Output>>>> + Send + 'static,
    > {
        // parse diemnet protocols
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
        let fut_socket = self.base_transport.dial(peer_id, base_addr)?;

        // outbound dial upgrade task
        let upgrade_fut = upgrade_outbound(self.ctxt.clone(), fut_socket, addr, peer_id, pubkey);
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
        let enable_proxy_protocol = self.enable_proxy_protocol;
        // stream of inbound upgrade tasks
        let inbounds = listener.map_ok(move |(fut_socket, addr)| {
            // inbound upgrade task
            let fut_upgrade = upgrade_inbound(
                ctxt.clone(),
                fut_socket,
                addr.clone(),
                enable_proxy_protocol,
            );
            let fut_upgrade = timeout_io(TRANSPORT_TIMEOUT, fut_upgrade);
            (fut_upgrade, addr)
        });

        Ok((inbounds, listen_addr))
    }
}

// If using `DiemNetTransport` as a `Transport` trait, then all upgrade futures
// and listening streams must be boxed, since `upgrade_inbound` and `upgrade_outbound`
// are async fns (and therefore unnamed types).
//
// TODO(philiphayes): We can change these `Pin<Box<dyn Future<..>>> to `impl Future<..>`
// when/if this rust feature is stabilized: https://github.com/rust-lang/rust/issues/63063

impl<TTransport: Transport> Transport for DiemNetTransport<TTransport>
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

    fn dial(&self, peer_id: PeerId, addr: NetworkAddress) -> io::Result<Self::Outbound> {
        self.dial(peer_id, addr)
            .map(|upgrade_fut| upgrade_fut.boxed())
    }

    fn listen_on(&self, addr: NetworkAddress) -> io::Result<(Self::Listener, NetworkAddress)> {
        let (listener, listen_addr) = self.listen_on(addr)?;
        let listener = listener
            .map_ok(|(upgrade_fut, addr)| (upgrade_fut.boxed(), addr))
            .boxed();
        Ok((listener, listen_addr))
    }
}
