// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! # An implementation of ProxyProtocol for HAProxy
//! https://www.haproxy.org/download/1.8/doc/proxy-protocol.txt
//!
//! ## Limitations
//! - Only supports ProxyProtocol V2
//! - Rejects ProxyProtocol V1
//! - Only supports IPv4 & IPv6 TCP mode
//! - All other valid connections, will just drop the address information and use the proxy's one
//! - Does not interpret TLVs
//!
//! ## Interpetations not in the spec
//! - An address space that doesn't match the size expected is rejected e.g. too big for IPv4
//! - Address space that's larger than the current supported requests is rejected

use diem_types::network_address::NetworkAddress;
use futures::io::{AsyncRead, AsyncReadExt};
use std::{
    convert::TryInto,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

/// Unique signature required at the beginning of every PPv2 connection
const PPV2_SIGNATURE: [u8; 12] = [
    0x0D, 0x0A, 0x0D, 0x0A, 0x00, 0x0D, 0x0A, 0x51, 0x55, 0x49, 0x54, 0x0A,
];

// Commands
/// Command for healthchecks and local traffic
const PPV2_LOCAL: u8 = 0x20;
/// Command for proxy traffic
const PPV2_PROXY: u8 = 0x21;

// Protocol families
const LOCAL_PROTOCOL: u8 = 0x00;
const TCP_IPV4: u8 = 0x11;
const TCP_IPV6: u8 = 0x21;
const TCP_UNIX: u8 = 0x31;
const UDP_IPV4: u8 = 0x12;
const UDP_IPV6: u8 = 0x22;
const UDP_UNIX: u8 = 0x32;

// Address sizes
const IPV4_SIZE: u16 = 12;
const IPV6_SIZE: u16 = 36;

/// Read a proxy protocol event and unwrap the address information associated.
pub async fn read_header<T: AsyncRead + std::marker::Unpin>(
    original_addr: &NetworkAddress,
    stream: &mut T,
) -> io::Result<NetworkAddress> {
    // This is small enough that it should not be fragmented by TCP
    let mut header = [0u8; 16];
    stream.read_exact(&mut header).await?;

    // If it's not proxy protocol, let's stop
    if header[0..12] != PPV2_SIGNATURE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "ProxyProtocol: Invalid signature",
        ));
    }

    // High 4 bits is version, low 4 bits is command
    let version_and_command = header[12];
    let _ = match version_and_command {
        PPV2_LOCAL | PPV2_PROXY => (),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "ProxyProtocol: Unsupported command or protocol version",
            ));
        }
    };

    // High 4 bits is family, low 4 bits is protocol
    let family_and_protocol = header[13];
    let address_size: [u8; 2] = header[14..16].try_into().unwrap();
    let address_size = u16::from_be_bytes(address_size);

    let mut address_bytes: Vec<u8> = vec![0; address_size as usize];
    stream.read_exact(&mut address_bytes).await?;

    let source_address = match family_and_protocol {
        // TODO: Support UDP in the future
        LOCAL_PROTOCOL | UDP_IPV4 | UDP_IPV6 | TCP_UNIX | UDP_UNIX => {
            // UNSPEC, UDP, and UNIX Steam/datagram
            // Accept connection but ignore address info as per spec
            original_addr.clone()
        }
        TCP_IPV4 => {
            // This is not mentioned in the spec, but if it doesn't match we might not read correctly
            if address_size < IPV4_SIZE {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "ProxyProtocol: Header size doesn't match expected address type",
                ));
            }

            let src_addr = u32::from_be_bytes(address_bytes[0..4].try_into().unwrap());
            let src_port = u16::from_be_bytes(address_bytes[8..10].try_into().unwrap());
            let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from(src_addr)), src_port);
            NetworkAddress::from(socket_addr)
        }
        TCP_IPV6 => {
            // This is not mentioned in the spec, but if it doesn't match we might not read correctly
            if address_size < IPV6_SIZE {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "ProxyProtocol: Header size doesn't match expected address type",
                ));
            }

            let src_addr = u128::from_be_bytes(address_bytes[0..16].try_into().unwrap());
            let src_port = u16::from_be_bytes(address_bytes[32..34].try_into().unwrap());

            let socket_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::from(src_addr)), src_port);
            NetworkAddress::from(socket_addr)
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "ProxyProtocol: Unsupported Address Family or Protocol",
            ));
        }
    };

    Ok(source_address)
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::{executor::block_on, future::join, io::AsyncWriteExt};
    use memsocket::MemorySocket;
    use std::net::ToSocketAddrs;

    const TEST_DATA: &[u8; 4] = &[0xDE, 0xAD, 0xBE, 0xEF];
    const IPV4_ADDR_1: &[u8; 4] = &[0x00, 0x00, 0x00, 0x01];
    const IPV4_ADDR_2: &[u8; 4] = &[0x00, 0x00, 0x00, 0x02];
    const IPV6_ADDR_1: &[u8; 16] = &[
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x01,
    ];
    const IPV6_ADDR_2: &[u8; 16] = &[
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x02,
    ];
    const PORT_80: &[u8; 2] = &[0x00, 80];
    const IPV4_ADDR_SIZE: &[u8; 2] = &[0x00, IPV4_SIZE as u8];
    const IPV6_ADDR_SIZE: &[u8; 2] = &[0x00, IPV6_SIZE as u8];

    async fn send_v4(sender: &mut MemorySocket) -> io::Result<()> {
        sender.write_all(&PPV2_SIGNATURE).await?; // V2 signature
        sender.write_all(&[PPV2_PROXY]).await?; // Version 2, Proxy
        sender.write_all(&[TCP_IPV4]).await?; // TCP IPv4
        sender.write_all(IPV4_ADDR_SIZE).await?; // Size of address
        sender.write_all(IPV4_ADDR_1).await?; // Sender address
        sender.write_all(IPV4_ADDR_2).await?; // Receiver address
        sender.write_all(PORT_80).await?; // Sender port
        sender.write_all(PORT_80).await?; // Receiver port
        sender.write_all(TEST_DATA).await // Data
    }

    async fn send_v6(sender: &mut MemorySocket) -> io::Result<()> {
        sender.write_all(&PPV2_SIGNATURE).await?; // V2 signature
        sender.write_all(&[PPV2_PROXY]).await?; // Version 2, Proxy
        sender.write_all(&[TCP_IPV6]).await?; // TCP IPv6
        sender.write_all(IPV6_ADDR_SIZE).await?; // Size of address
        sender.write_all(IPV6_ADDR_1).await?; // Sender address
        sender.write_all(IPV6_ADDR_2).await?; // Receiver address
        sender.write_all(PORT_80).await?; // Sender port
        sender.write_all(PORT_80).await?; // Receiver port
        sender.write_all(TEST_DATA).await // Data
    }

    #[test]
    fn test_ipv4_proxy_protocol() {
        let (mut sender, mut receiver) = MemorySocket::new_pair();
        let original_addr = NetworkAddress::mock();

        let server = async move {
            send_v4(&mut sender).await.expect("Successful send");
        };

        let reader = read_header(&original_addr, &mut receiver);

        // Verify the correct address is picked up
        let client = async move {
            let addr = reader.await.expect("An address");
            let addresses: Vec<_> = addr.to_socket_addrs().unwrap().collect();
            let socket_addr = addresses.first().unwrap();
            assert_eq!(IpAddr::V4(Ipv4Addr::from(0x0000000001)), socket_addr.ip());
            assert_eq!(80, socket_addr.port());
        };

        block_on(join(server, client));

        // Check that data wrapped isn't affected
        let check_data = async move {
            let data: &mut [u8; 4] = &mut [0; 4];
            receiver.read_exact(data).await.unwrap();
            assert_eq!(TEST_DATA, data);
        };

        block_on(check_data);
    }

    #[test]
    fn test_ipv6_proxy_protocol() {
        let (mut sender, mut receiver) = MemorySocket::new_pair();
        let original_addr = NetworkAddress::mock();

        let server = async move {
            send_v6(&mut sender).await.expect("Successful send");
        };

        let reader = read_header(&original_addr, &mut receiver);
        // Verify the correct address is picked up
        let client = async move {
            let addr = reader.await.expect("An address");
            let addresses: Vec<_> = addr.to_socket_addrs().unwrap().collect();
            let socket_addr = addresses.first().unwrap();
            assert_eq!(
                IpAddr::V6(Ipv6Addr::from(0x00000001000000010000000100000001)),
                socket_addr.ip()
            );
            assert_eq!(80, socket_addr.port());
        };

        block_on(join(server, client));

        // Check that data wrapped isn't affected
        let check_data = async move {
            let data: &mut [u8; 4] = &mut [0; 4];
            receiver.read_exact(data).await.unwrap();
            assert_eq!(TEST_DATA, data);
        };

        block_on(check_data);
    }

    #[test]
    fn test_local_proxy_protocol() {
        let address_bytes: [&[u8; 1]; 5] = [
            &[LOCAL_PROTOCOL],
            &[UDP_IPV4],
            &[UDP_IPV6],
            &[TCP_UNIX],
            &[UDP_UNIX],
        ];

        address_bytes
            .iter()
            .for_each(|addr| test_skip_address(addr));
    }

    fn test_skip_address(address_byte: &[u8; 1]) {
        let (mut sender, mut receiver) = MemorySocket::new_pair();
        let original_addr = NetworkAddress::mock();

        let server = async move {
            sender.write_all(&PPV2_SIGNATURE).await.unwrap(); // V2 signature
            sender.write_all(&[PPV2_PROXY]).await.unwrap(); // Version 2, Proxy
            sender.write_all(address_byte).await.unwrap(); // Local, don't use address
            sender.write_all(IPV4_ADDR_SIZE).await.unwrap(); // Size of address
            sender.write_all(IPV4_ADDR_1).await.unwrap(); // Sender address
            sender.write_all(IPV4_ADDR_2).await.unwrap(); // Receiver address
            sender.write_all(PORT_80).await.unwrap(); // Sender port
            sender.write_all(PORT_80).await.unwrap(); // Receiver port
            sender.write_all(TEST_DATA).await.unwrap() // Data
        };

        let reader = read_header(&original_addr, &mut receiver);
        let proxy_addr = original_addr.clone();
        // Verify no change in the address
        let client = async move {
            let addr = reader.await.expect("An address");
            assert_eq!(addr, proxy_addr.clone());
        };

        block_on(join(server, client));

        // Check that data wrapped isn't affected
        let check_data = async move {
            let data: &mut [u8; 4] = &mut [0; 4];
            receiver.read_exact(data).await.unwrap();
            assert_eq!(TEST_DATA, data);
        };

        block_on(check_data);
    }

    #[test]
    fn test_error_handling() {
        // Bad Header
        test_error_case(&[&[0; 12]]);

        // Bad Version
        test_error_case(&[&PPV2_SIGNATURE, &[0x00]]);

        // Bad Command
        test_error_case(&[&PPV2_SIGNATURE, &[0x22]]);

        // Invalid address type
        test_error_case(&[&PPV2_SIGNATURE, &[PPV2_PROXY], &[0x55], &[0x00, 0x00]]);

        // Invalid size mismatch IPv4 / IPv6
        test_error_case(&[&PPV2_SIGNATURE, &[PPV2_PROXY], &[TCP_IPV4], &[0x00, 0x00]]);
        test_error_case(&[&PPV2_SIGNATURE, &[PPV2_PROXY], &[TCP_IPV6], &[0x00, 0x00]]);
    }

    fn test_error_case(input: &[&[u8]]) {
        let (mut sender, mut receiver) = MemorySocket::new_pair();
        let original_addr = NetworkAddress::mock();

        let server = async move {
            for array in input {
                sender.write_all(array).await.expect("Successful send");
            }
        };

        let reader = read_header(&original_addr, &mut receiver);
        // Bad header should fail
        let client = async move {
            reader.await.expect_err("Expected error");
        };

        block_on(join(server, client));
    }
}
