// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::DiscoveryError;
use diem_config::config::PeerSet;
use diem_time_service::{Interval, TimeService, TimeServiceTrait};
use futures::Stream;
use std::{
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

pub struct FileStream {
    file_path: PathBuf,
    interval: Pin<Box<Interval>>,
}

impl FileStream {
    pub(crate) fn new(
        file_path: &Path,
        interval_duration: Duration,
        time_service: TimeService,
    ) -> Self {
        FileStream {
            file_path: file_path.to_path_buf(),
            interval: Box::pin(time_service.interval(interval_duration)),
        }
    }
}

impl Stream for FileStream {
    type Item = Result<PeerSet, DiscoveryError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Wait for delay, or add the delay for next call
        futures::ready!(self.interval.as_mut().poll_next(cx));

        Poll::Ready(Some(match load_file(self.file_path.as_path()) {
            Ok(peers) => Ok(peers),
            Err(error) => Err(error),
        }))
    }
}

/// Loads a YAML configuration file
fn load_file(path: &Path) -> Result<PeerSet, DiscoveryError> {
    let contents = std::fs::read_to_string(path).map_err(DiscoveryError::IO)?;
    serde_yaml::from_str(&contents).map_err(|err| DiscoveryError::Parsing(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DiscoveryChangeListener;
    use channel::Receiver;
    use diem_config::{
        config::{Peer, PeerRole},
        network_id::NetworkContext,
    };
    use diem_temppath::TempPath;
    use diem_types::{network_address::NetworkAddress, PeerId};
    use futures::StreamExt;
    use network::connectivity_manager::{ConnectivityRequest, DiscoverySource};
    use std::{collections::HashSet, str::FromStr, sync::Arc};
    use tokio::time::sleep;

    fn create_listener(path: Arc<TempPath>) -> Receiver<ConnectivityRequest> {
        let check_interval = Duration::from_millis(5);
        // TODO: Figure out why mock time doesn't work right
        let time_service = TimeService::real();
        let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) =
            channel::new(1, &network::counters::PENDING_CONNECTIVITY_MANAGER_REQUESTS);
        let listener_task = async move {
            let listener = DiscoveryChangeListener::file(
                NetworkContext::mock(),
                conn_mgr_reqs_tx,
                path.as_ref().as_ref(),
                check_interval,
                time_service,
            );
            Box::pin(listener).run().await
        };

        tokio::task::spawn(listener_task);
        conn_mgr_reqs_rx
    }

    fn write_peer_set(peers: &PeerSet, path: &Path) {
        let file_contents = serde_yaml::to_vec(peers).unwrap();
        std::fs::write(path, file_contents).unwrap();
    }

    #[tokio::test]
    async fn test_file_listener() {
        let path = TempPath::new();
        path.create_as_file().unwrap();
        let path = Arc::new(path);

        let peers: PeerSet = PeerSet::new();
        write_peer_set(&peers, path.as_ref().as_ref());

        let mut conn_mgr_reqs_rx = create_listener(path.clone());

        // Try empty
        if let Some(ConnectivityRequest::UpdateDiscoveredPeers(
            DiscoverySource::File,
            actual_peers,
        )) = conn_mgr_reqs_rx.next().await
        {
            assert_eq!(peers, actual_peers)
        } else {
            panic!("No message sent by discovery")
        }

        // Try with a peer
        let mut peers = PeerSet::new();
        let addr = NetworkAddress::from_str("/ip4/1.2.3.4/tcp/6180/ln-noise-ik/080e287879c918794170e258bfaddd75acac5b3e350419044655e4983a487120/ln-handshake/0").unwrap();
        let key = addr.find_noise_proto().unwrap();
        let addrs = vec![addr];
        let mut keys = HashSet::new();
        keys.insert(key);
        peers.insert(
            PeerId::random(),
            Peer::new(addrs, keys, PeerRole::Downstream),
        );
        let file_contents = serde_yaml::to_vec(&peers).unwrap();
        std::fs::write(path.as_ref(), file_contents).unwrap();
        // Clear old items
        while conn_mgr_reqs_rx.next().await.is_none() {}

        if let Some(ConnectivityRequest::UpdateDiscoveredPeers(
            DiscoverySource::File,
            actual_peers,
        )) = conn_mgr_reqs_rx.next().await
        {
            assert_eq!(peers, actual_peers)
        } else {
            panic!("No message sent by discovery")
        }
    }

    #[tokio::test]
    async fn test_no_file() {
        let path = TempPath::new();
        path.create_as_file().unwrap();
        let path = Arc::new(path);

        let mut conn_mgr_reqs_rx = create_listener(path.clone());
        let writer_task = async move {
            sleep(Duration::from_secs(1)).await;

            let peers: PeerSet = PeerSet::new();
            write_peer_set(&peers, path.as_ref().as_ref());
        };
        tokio::task::spawn(writer_task);

        if let Some(ConnectivityRequest::UpdateDiscoveredPeers(
            DiscoverySource::File,
            actual_peers,
        )) = conn_mgr_reqs_rx.next().await
        {
            assert_eq!(PeerSet::new(), actual_peers)
        } else {
            panic!("No message sent by discovery")
        }
    }
}
