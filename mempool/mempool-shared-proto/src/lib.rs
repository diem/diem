//! Proto crate for shared mempool

pub mod proto;
use crate::proto::mempool_status::MempoolAddTransactionStatusCode;
use failure::prelude::*;
use proto_conv::{FromProto, IntoProto};

/// Status of transaction insertion operation
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MempoolAddTransactionStatus {
    /// Status code of the transaction insertion operation
    pub code: MempoolAddTransactionStatusCode,
    /// Message to give more details about the transaction insertion operation
    pub message: String,
}

impl MempoolAddTransactionStatus {
    /// Create a new MempoolAddTransactionStatus
    pub fn new(code: MempoolAddTransactionStatusCode, message: String) -> Self {
        Self { code, message }
    }
}

//***********************************
// Decoding/Encoding to Protobuffers
//***********************************
impl IntoProto for MempoolAddTransactionStatus {
    type ProtoType = proto::mempool_status::MempoolAddTransactionStatus;

    fn into_proto(self) -> Self::ProtoType {
        let mut mempool_add_transaction_status = Self::ProtoType::new();
        mempool_add_transaction_status.set_message(self.message);
        mempool_add_transaction_status.set_code(self.code);
        mempool_add_transaction_status
    }
}

impl FromProto for MempoolAddTransactionStatus {
    type ProtoType = proto::mempool_status::MempoolAddTransactionStatus;

    fn from_proto(proto: Self::ProtoType) -> Result<Self> {
        Ok(MempoolAddTransactionStatus::new(
            proto.get_code(),
            proto.get_message().to_string(),
        ))
    }
}
