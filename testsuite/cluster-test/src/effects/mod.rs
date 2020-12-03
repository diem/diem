// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;
use async_trait::async_trait;
use futures::future::try_join_all;
use std::fmt::Display;

pub mod network_delay;
pub mod packet_loss;
pub mod stop_validator;

#[async_trait]
pub trait Effect: Display {
    async fn activate(&mut self) -> Result<()>;
    async fn deactivate(&mut self) -> Result<()>;
}

pub async fn activate_all<T: Effect>(effects: &mut Vec<T>) -> Result<()> {
    try_join_all(effects.iter_mut().map(Effect::activate)).await?;
    Ok(())
}

pub async fn deactivate_all<T: Effect>(effects: &mut Vec<T>) -> Result<()> {
    try_join_all(effects.iter_mut().map(Effect::deactivate)).await?;
    Ok(())
}
