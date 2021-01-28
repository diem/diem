// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::HashValue;
use diem_framework_releases::legacy::transaction_scripts;
use diem_json_rpc_types::views::{
    BytesView, MoveAbortExplanationView, ScriptView, TransactionDataView, VMStatusView,
};
use diem_types::{
    transaction::{Script, ScriptFunction, Transaction, TransactionArgument, TransactionPayload},
    vm_status::{AbortLocation, KeptVMStatus},
};
use move_core_types::language_storage::{StructTag, TypeTag};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{convert::TryFrom, fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SdkLang {
    Rust,
    Java,
    Python,
    Typescript,
    Go,
    CSharp,
    Cpp,
    Unknown,
}

impl Default for SdkLang {
    fn default() -> Self {
        SdkLang::Unknown
    }
}

impl SdkLang {
    pub fn from_str(user_agent_part: &str) -> SdkLang {
        match str::trim(user_agent_part) {
            "diem-client-sdk-rust" => SdkLang::Rust,
            "diem-client-sdk-java" => SdkLang::Java,
            "diem-client-sdk-python" => SdkLang::Python,
            "diem-client-sdk-typescript" => SdkLang::Typescript,
            "diem-client-sdk-golang" => SdkLang::Go,
            "diem-client-sdk-csharp" => SdkLang::CSharp,
            "diem-client-sdk-cpp" => SdkLang::Cpp,
            _ => SdkLang::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            SdkLang::Rust => "rust",
            SdkLang::Java => "java",
            SdkLang::Python => "python",
            SdkLang::Typescript => "typescript",
            SdkLang::Go => "golang",
            SdkLang::CSharp => "csharp",
            SdkLang::Cpp => "cpp",
            SdkLang::Unknown => "unknown",
        }
    }
}

static SDK_VERSION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b([0-3])\.(\d{1,2})\.(\d{1,2})\b").unwrap());

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SdkVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl SdkVersion {
    pub fn from_str(user_agent_part: &str) -> SdkVersion {
        if let Some(captures) = SDK_VERSION_REGEX.captures(user_agent_part) {
            if captures.len() == 4 {
                if let (Ok(major), Ok(minor), Ok(patch)) = (
                    u16::from_str(&captures[1]),
                    u16::from_str(&captures[2]),
                    u16::from_str(&captures[3]),
                ) {
                    return SdkVersion {
                        major,
                        minor,
                        patch,
                    };
                }
            }
        }
        SdkVersion::default()
    }
}

impl fmt::Display for SdkVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct SdkInfo {
    pub language: SdkLang,
    pub version: SdkVersion,
}

impl SdkInfo {
    pub fn from_user_agent(user_agent: &str) -> SdkInfo {
        // parse our sdk user agent strings, i.e `diem-client-sdk-python / 0.1.12`
        let lowercase_user_agent = user_agent.to_lowercase();
        let user_agent_parts: Vec<&str> = lowercase_user_agent.split('/').collect();
        if user_agent_parts.len() == 2 {
            let language = SdkLang::from_str(&user_agent_parts[0]);
            let version = SdkVersion::from_str(&user_agent_parts[1]);
            if language != SdkLang::Unknown && version != SdkVersion::default() {
                return SdkInfo { language, version };
            }
        }
        SdkInfo::default()
    }
}

pub fn vm_status_view_from_kept_vm_status(status: &KeptVMStatus) -> VMStatusView {
    match status {
        KeptVMStatus::Executed => VMStatusView::Executed,
        KeptVMStatus::OutOfGas => VMStatusView::OutOfGas,
        KeptVMStatus::MoveAbort(loc, abort_code) => {
            let explanation = if let AbortLocation::Module(module_id) = loc {
                move_explain::get_explanation(module_id, *abort_code).map(|context| {
                    MoveAbortExplanationView {
                        category: context.category.code_name,
                        category_description: context.category.code_description,
                        reason: context.reason.code_name,
                        reason_description: context.reason.code_description,
                    }
                })
            } else {
                None
            };

            VMStatusView::MoveAbort {
                explanation,
                location: loc.to_string(),
                abort_code: *abort_code,
            }
        }
        KeptVMStatus::ExecutionFailure {
            location,
            function,
            code_offset,
        } => VMStatusView::ExecutionFailure {
            location: location.to_string(),
            function_index: *function,
            code_offset: *code_offset,
        },
        KeptVMStatus::MiscellaneousError => VMStatusView::MiscellaneousError,
    }
}

pub fn transaction_data_view_from_transaction(tx: Transaction) -> TransactionDataView {
    match tx {
        Transaction::BlockMetadata(t) => TransactionDataView::BlockMetadata {
            timestamp_usecs: t.timestamp_usec(),
        },
        Transaction::GenesisTransaction(_) => TransactionDataView::WriteSet {},
        Transaction::UserTransaction(t) => {
            let script_hash = match t.payload() {
                TransactionPayload::Script(s) => HashValue::sha3_256_of(s.code()),
                _ => HashValue::zero(),
            };

            let script_bytes: BytesView = match t.payload() {
                TransactionPayload::Script(s) => bcs::to_bytes(s).unwrap_or_default(),
                TransactionPayload::ScriptFunction(s) => bcs::to_bytes(s).unwrap_or_default(),
                _ => vec![],
            }
            .into();

            let script: ScriptView = match t.payload() {
                TransactionPayload::Script(s) => script_view_from_script(s),
                TransactionPayload::ScriptFunction(s) => script_view_from_script_function(s),
                _ => ScriptView::unknown(),
            };

            TransactionDataView::UserTransaction {
                sender: t.sender(),
                signature_scheme: t.authenticator().sender().scheme().to_string(),
                signature: t.authenticator().sender().signature_bytes().into(),
                public_key: t.authenticator().sender().public_key_bytes().into(),
                secondary_signers: Some(t.authenticator().secondary_signer_addreses()),
                secondary_signature_schemes: Some(
                    t.authenticator()
                        .secondary_signers()
                        .iter()
                        .map(|account_auth| account_auth.scheme().to_string())
                        .collect(),
                ),
                secondary_signatures: Some(
                    t.authenticator()
                        .secondary_signers()
                        .iter()
                        .map(|account_auth| account_auth.signature_bytes().into())
                        .collect(),
                ),
                secondary_public_keys: Some(
                    t.authenticator()
                        .secondary_signers()
                        .iter()
                        .map(|account_auth| account_auth.public_key_bytes().into())
                        .collect(),
                ),
                sequence_number: t.sequence_number(),
                chain_id: t.chain_id().id(),
                max_gas_amount: t.max_gas_amount(),
                gas_unit_price: t.gas_unit_price(),
                gas_currency: t.gas_currency_code().to_string(),
                expiration_timestamp_secs: t.expiration_timestamp_secs(),
                script_hash,
                script_bytes,
                script,
            }
        }
    }
}

pub fn script_view_from_script(script: &Script) -> ScriptView {
    let name = transaction_scripts::LegacyStdlibScript::try_from(script.code())
        .map(|x| format!("{}", x))
        .unwrap_or_else(|_| "unknown".to_string());
    let ty_args: Vec<String> = script
        .ty_args()
        .iter()
        .map(|type_tag| match type_tag {
            TypeTag::Struct(StructTag { module, .. }) => module.to_string(),
            tag => format!("{}", tag),
        })
        .collect();
    let mut view = ScriptView {
        r#type: name.clone(),
        code: Some(script.code().into()),
        arguments: Some(
            script
                .args()
                .iter()
                .map(|arg| format!("{:?}", &arg))
                .collect(),
        ),
        type_arguments: Some(ty_args.clone()),
        ..Default::default()
    };

    // handle legacy fields, backward compatible
    if name == "peer_to_peer_with_metadata" {
        if let [TransactionArgument::Address(receiver), TransactionArgument::U64(amount), TransactionArgument::U8Vector(metadata), TransactionArgument::U8Vector(metadata_signature)] =
            script.args()
        {
            view.receiver = Some(*receiver);
            view.amount = Some(*amount);
            view.currency = Some(
                ty_args
                    .get(0)
                    .unwrap_or(&"unknown_currency".to_string())
                    .to_string(),
            );
            view.metadata = Some(BytesView::new(metadata.as_ref()));
            view.metadata_signature = Some(BytesView::new(metadata_signature.as_ref()));
        }
    }

    view
}

pub fn script_view_from_script_function(script: &ScriptFunction) -> ScriptView {
    let ty_args: Vec<String> = script
        .ty_args()
        .iter()
        .map(|type_tag| match type_tag {
            TypeTag::Struct(StructTag { module, .. }) => module.to_string(),
            tag => format!("{}", tag),
        })
        .collect();
    ScriptView {
        r#type: "script_function".to_string(),
        module_address: Some(*script.module().address()),
        module_name: Some(script.module().name().to_string()),
        function_name: Some(script.function().to_string()),
        arguments_bcs: Some(
            script
                .args()
                .iter()
                .map(|arg| BytesView::from(arg.as_ref()))
                .collect(),
        ),
        type_arguments: Some(ty_args),
        ..Default::default()
    }
}

pub fn sdk_info_from_user_agent(user_agent: Option<&str>) -> SdkInfo {
    match user_agent {
        Some(user_agent) => SdkInfo::from_user_agent(user_agent),
        None => SdkInfo::default(),
    }
}
