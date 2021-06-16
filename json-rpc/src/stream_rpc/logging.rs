// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_logger::Schema;
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamRpcAction<'a> {
    ClientConnectionLog(ClientConnectionLog<'a>),
    HttpRequestLog(HttpRequestLog<'a>),
}

#[derive(Schema)]
pub struct StreamRpcLog<'a> {
    pub transport: &'static str,
    pub user_agent: Option<&'a str>,
    pub remote_addr: Option<&'a str>,

    pub action: StreamRpcAction<'a>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
#[derive(Schema)]
pub struct HttpRequestLog<'a> {
    pub path: &'a str,
    pub status: u16,
    pub referer: Option<&'a str>,
    pub forwarded: Option<&'a str>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
#[derive(Schema)]
pub struct ClientConnectionLog<'a> {
    pub client_id: Option<u64>,
    pub forwarded: Option<&'a str>,
    pub rpc_method: Option<&'static str>,
}
