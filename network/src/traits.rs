use crate::{
    connectivity_manager::ConnectivityRequest,
    peer_manager::{
        ConnectionRequest, ConnectionRequestSender, PeerManagerRequest, PeerManagerRequestSender,
    },
    ProtocolId,
};
use channel::libra_channel;
use libra_types::PeerId;

pub trait FromPeerManagerAndConnectionRequestSenders
where
    Self: std::marker::Sized,
{
    fn from_peer_manager_and_connection_request_senders(
        pm_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
    ) -> Self;

    fn from_libra_channel_senders(
        pm_reqs_tx: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
        connection_reqs_tx: libra_channel::Sender<PeerId, ConnectionRequest>,
    ) -> Self {
        Self::from_peer_manager_and_connection_request_senders(
            PeerManagerRequestSender::new(pm_reqs_tx),
            ConnectionRequestSender::new(connection_reqs_tx),
        )
    }
}

pub trait FromPeerManagerConnectionConnectivityRequestSenders
where
    Self: std::marker::Sized,
{
    fn from_peer_manager_connection_request_connection_manager_senders(
        pm_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
        connectivity_reqs_tx: channel::Sender<ConnectivityRequest>,
    ) -> Self;

    fn from_channel_senders(
        pm_reqs_tx: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
        connection_reqs_tx: libra_channel::Sender<PeerId, ConnectionRequest>,
        connectivity_reqs_tx: channel::Sender<ConnectivityRequest>,
    ) -> Self {
        Self::from_peer_manager_connection_request_connection_manager_senders(
            PeerManagerRequestSender::new(pm_reqs_tx),
            ConnectionRequestSender::new(connection_reqs_tx),
            connectivity_reqs_tx,
        )
    }
}
