/// Implements a simple LibraNet client
use crate::protocols::identity::{sync_exchange_handshake, sync_peer_id_exchange};
use crate::protocols::wire::{handshake::v1::*, messaging::v1::*};
use bytes::BytesMut;
use libra_types::PeerId;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    io::{self, Read, Result, Write},
    net::{TcpStream, ToSocketAddrs},
};

#[allow(unused)]
struct LibraNetClient<TSocket> {
    protocol: ProtocolId,
    peer_id: PeerId,
    socket: TSocket,
    next_request_id: RequestId,
}

impl LibraNetClient<TcpStream> {
    #[allow(unused)]
    fn connect<A: ToSocketAddrs>(
        addr: A,
        protocol: ProtocolId,
    ) -> Result<LibraNetClient<TcpStream>> {
        let socket = TcpStream::connect(addr)?;
        LibraNetClient::new(socket, protocol)
    }
}

impl<TSocket> LibraNetClient<TSocket>
where
    TSocket: Read + Write,
{
    #[allow(unused)]
    // Receives a socket and performs the handshake protocol over it.
    fn new(mut socket: TSocket, protocol: ProtocolId) -> Result<LibraNetClient<TSocket>> {
        let supported_protocols = [protocol].iter().into();
        let peer_id = PeerId::random();
        let mut self_handshake = HandshakeMsg::new();
        self_handshake.add(MessagingProtocolVersion::V1, supported_protocols);
        let remote_peer_id = sync_peer_id_exchange(&peer_id, &mut socket)?;
        let remote_handshake = sync_exchange_handshake(&self_handshake, &mut socket)?;
        if let None = remote_handshake.find_common_protocols(&self_handshake) {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Protocol not supported",
            ));
        }
        Ok(Self {
            peer_id: remote_peer_id,
            socket,
            protocol,
            next_request_id: 0,
        })
    }

    #[allow(unused)]
    fn send_rpc<Message>(&mut self, req: &Message) -> Result<Message>
    where
        Message: Serialize + DeserializeOwned,
    {
        let request_id = self.next_request_id;
        self.next_request_id = request_id + 1;
        // write request
        let raw_request = lcs::to_bytes(req)
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to serialize raw request bytes: {}", e),
                )
            })?
            .into();
        let rpc_request = RpcRequest {
            request_id,
            protocol_id: self.protocol,
            priority: 0,
            raw_request,
        };
        let raw_rpc_request =
            lcs::to_bytes(&NetworkMessage::RpcRequest(rpc_request)).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to serialize RpcRequest msg: {}", e),
                )
            })?;
        framing::write_u32frame(&mut self.socket, &raw_rpc_request)?;
        // read response
        let mut raw_rpc_response = BytesMut::new();
        framing::read_u32frame(&mut self.socket, &mut raw_rpc_response)?;
        let msg: NetworkMessage = lcs::from_bytes(&raw_rpc_response).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to deserialize RpcResponse msg: {}", e),
            )
        })?;
        if let NetworkMessage::RpcResponse(rpc_response) = msg {
            assert_eq!(rpc_response.request_id, request_id);
            let response: Message = lcs::from_bytes(&rpc_response.raw_response).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to deserialize raw response bytes: {}", e),
                )
            })?;
            Ok(response)
        } else {
            Err(io::Error::from(io::ErrorKind::InvalidData))
        }
    }
}

mod framing {
    use bytes::BytesMut;
    use std::{
        convert::TryInto,
        io::{Read, Result, Write},
    };

    pub(super) fn read_u32frame<'stream, 'buf, 'c, TSocket>(
        mut stream: &'stream mut TSocket,
        buf: &'buf mut BytesMut,
    ) -> Result<()>
    where
        'stream: 'c,
        'buf: 'c,
        TSocket: Read,
    {
        let len = read_u32frame_len(&mut stream)?;
        buf.resize(len as usize, 0);
        stream.read_exact(buf.as_mut())?;
        Ok(())
    }

    /// Read a u32 (encoded as BE bytes) from `Stream` and return the length.
    fn read_u32frame_len<TSocket>(stream: &mut TSocket) -> Result<u32>
    where
        TSocket: Read,
    {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf)?;
        Ok(u32::from_be_bytes(len_buf))
    }

    /// Write the contents of `buf` to `stream` prefixed with a u32 length.
    /// The length of `buf` must be less than or equal to u32::max_value().
    ///
    /// Caller is responsible for flushing the write to `stream`.
    pub(super) fn write_u32frame<'stream, 'buf, 'c, TSocket>(
        mut stream: &'stream mut TSocket,
        buf: &'buf [u8],
    ) -> Result<()>
    where
        'stream: 'c,
        'buf: 'c,
        TSocket: Write,
    {
        let len = buf
            .len()
            .try_into()
            // TODO Maybe use our own Error Type?
            .map_err(|_e| std::io::Error::new(std::io::ErrorKind::Other, "Too big"))?;
        write_u32frame_len(&mut stream, len)?;
        stream.write_all(buf)?;
        Ok(())
    }

    /// Write a u32 `len` as BE bytes to `stream`.
    ///
    /// Caller is responsible for flushing the write to `stream`.
    fn write_u32frame_len<TSocket>(stream: &mut TSocket, len: u32) -> Result<()>
    where
        TSocket: Write,
    {
        let len = u32::to_be_bytes(len);
        stream.write_all(&len)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::LibraNetClient;
    use crate::protocols::network::{dummy, Event};
    use futures::{future::join, StreamExt};
    use libra_logger::prelude::*;
    use netcore::transport::tcp::multiaddr_to_socketaddr;
    use std::time::Duration;

    #[test]
    fn test_sync_rpc() {
        ::libra_logger::Logger::new().environment_only(true).init();
        let mut tn = dummy::setup_network();
        let listener_peer_id = tn.listener_peer_id;
        let mut listener_events = tn.listener_events;
        let mut listener_sender = tn.listener_sender;
        let listener_addr = tn.listener_addr;

        let msg = dummy::DummyMsg(vec![]);
        let msg_clone = msg.clone();

        let f_respond = async move {
            // Wait for NewPeer event.
            let event = listener_events.next().await.unwrap().unwrap();
            assert!(matches!(event, Event::NewPeer(_)));
            // Wait for Rpc Request and send response.
            let event = listener_events.next().await.unwrap().unwrap();
            match event {
                Event::RpcRequest((peer_id, msg, rs)) => {
                    assert_eq!(msg, msg_clone);
                    rs.send(Ok(lcs::to_bytes(&msg).unwrap().into())).unwrap();
                }
                event => panic!("Unexpected event: {:?}", event),
            }
        };
        tn.runtime.spawn(f_respond);

        // Dialer send rpc request and receives rpc response
        let mut dialer = LibraNetClient::connect(
            multiaddr_to_socketaddr(&listener_addr).unwrap(),
            dummy::TEST_RPC_PROTOCOL,
        )
        .unwrap();
        let res_msg = dialer.send_rpc(&msg).unwrap();
        assert_eq!(res_msg, msg);
    }
}
