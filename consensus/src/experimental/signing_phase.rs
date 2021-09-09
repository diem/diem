// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use safety_rules::TSafetyRules;
use std::{
    fmt::{Debug, Display, Formatter},
    sync::Arc,
};

use crate::{
    experimental::pipeline_phase::{ResponseWithInstruction, StatelessPipeline},
    metrics_safety_rules::MetricsSafetyRules,
};
use async_trait::async_trait;
use diem_crypto::ed25519::Ed25519Signature;
use diem_infallible::Mutex;
use diem_types::ledger_info::{LedgerInfo, LedgerInfoWithSignatures};
use safety_rules::Error;

/// [ This class is used when consensus.decoupled = true ]
/// SigningPhase is a singleton that receives executed blocks from
/// the buffer manager and sign them. After getting the signature from
/// the safety rule, SigningPhase sends the signature and error (if any) back.

pub struct SigningRequest {
    pub ordered_ledger_info: LedgerInfoWithSignatures,
    pub commit_ledger_info: LedgerInfo,
}

impl Debug for SigningRequest {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for SigningRequest {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "SigningRequest({}, {})",
            self.ordered_ledger_info, self.commit_ledger_info
        )
    }
}

pub type SigningResponse = Result<Ed25519Signature, Error>;

pub struct SigningPhase {
    safety_rule_handle: Arc<Mutex<MetricsSafetyRules>>,
}

impl SigningPhase {
    pub fn new(safety_rule_handle: Arc<Mutex<MetricsSafetyRules>>) -> Self {
        Self { safety_rule_handle }
    }
}

#[async_trait]
impl StatelessPipeline for SigningPhase {
    type Request = SigningRequest;
    type Response = SigningResponse;
    async fn process(&self, req: SigningRequest) -> ResponseWithInstruction<SigningResponse> {
        let SigningRequest {
            ordered_ledger_info,
            commit_ledger_info,
        } = req;

        ResponseWithInstruction::from(
            self.safety_rule_handle
                .lock()
                .sign_commit_vote(ordered_ledger_info, commit_ledger_info),
        )
    }
}
