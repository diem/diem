// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_logger::debug;

#[derive(Clone)]
pub enum Transport {
    Websocket,
    SSE,
}

impl Transport {
    pub fn as_str(&self) -> &'static str {
        match self {
            Transport::Websocket => "websocket",
            Transport::SSE => "sse",
        }
    }
}

impl std::fmt::Debug for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub fn get_remote_addr(
    headers: &warp::http::HeaderMap,
    remote_addr: Option<&std::net::SocketAddr>,
) -> Option<String> {
    match headers.get(warp::http::header::FORWARDED) {
        Some(forward) => match forward.to_str() {
            Ok(forward) => Some(forward.to_string()),
            // A lot of things external to the node would have to go wrong for this to ever occur
            // Or the node is fully exposed to 'creative' clients passing their own `FORWARDED` header
            Err(e) => {
                debug!("Unable to parse 'FORWARDED' header to string, falling back to remote_addr '{:?}', {}", &remote_addr, e);
                remote_addr.map(|v| v.to_string())
            }
        },
        None => remote_addr.map(|v| v.to_string()),
    }
}

#[test]
fn test_get_remote_addr() {
    let socket = std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(7, 7, 7, 7)),
        7777,
    );
    let socket_result = Some("7.7.7.7:7777".to_string());
    assert_eq!(Some(socket.to_string()), socket_result);

    let headers_without_forwarded = warp::http::HeaderMap::new();
    assert_eq!(
        get_remote_addr(&headers_without_forwarded, Some(&socket)),
        socket_result
    );
    assert_eq!(get_remote_addr(&headers_without_forwarded, None), None);

    let forwarded_value = "forwarded".to_string();
    let forwarded_header = warp::http::header::HeaderValue::from_str(&forwarded_value).unwrap();
    let forwarded_result = Some(forwarded_value);
    assert_eq!(
        Some(forwarded_header.to_str().unwrap().to_string()),
        forwarded_result
    );

    let mut headers_with_forwarded = warp::http::HeaderMap::new();
    headers_with_forwarded.insert(warp::http::header::FORWARDED, forwarded_header);
    assert_eq!(
        get_remote_addr(&headers_with_forwarded, None),
        forwarded_result
    );
    assert_eq!(
        get_remote_addr(&headers_with_forwarded, Some(&socket)),
        forwarded_result
    );

    // someone is being a naughty
    let mut headers_with_invalid_forwarded = warp::http::HeaderMap::new();
    headers_with_invalid_forwarded.insert(
        warp::http::header::FORWARDED,
        warp::http::header::HeaderValue::from_str("хулиган").unwrap(),
    );
    assert_eq!(get_remote_addr(&headers_with_invalid_forwarded, None), None);
    assert_eq!(
        get_remote_addr(&headers_with_invalid_forwarded, Some(&socket)),
        socket_result
    );
}
