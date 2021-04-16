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
pub struct WebsocketDisconnect<'a> {
    pub remote_addr: Option<std::net::SocketAddr>,
    pub user_agent: &'a str,
    pub forwarded: Option<&'a str>,
}

#[derive(Schema)]
pub struct ClientConnect<'a> {
    pub transport: &'static str,
    pub remote_addr: Option<&'a str>,
    pub client_id: u64,
}

#[derive(Schema)]
pub struct ClientDisconnect<'a> {
    pub transport: &'static str,
    pub remote_addr: Option<&'a str>,
    pub client_id: u64,
}

#[derive(Schema)]
pub struct SubscriptionRequestLog<'a> {
    #[schema(display)]
    pub transport: &'static str,
    pub remote_addr: Option<&'a str>,
    pub client_id: u64,
    pub rpc_method: Option<&'static str>,
}
