// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_logger::Schema;

#[derive(Schema)]
pub struct HttpRequestLog<'a> {
    #[schema(display)]
    pub remote_addr: Option<std::net::SocketAddr>,
    pub path: &'a str,
    pub status: u16,
    pub referer: Option<&'a str>,
    pub user_agent: &'a str,
    pub forwarded: Option<&'a str>,
}

#[derive(Schema)]
pub struct ClientConnectionLog<'a> {
    pub transport: &'static str,
    pub remote_addr: Option<&'a str>,
    pub client_id: Option<u64>,
    pub user_agent: Option<&'a str>,
    pub forwarded: Option<&'a str>,
    pub rpc_method: Option<&'static str>,
}
