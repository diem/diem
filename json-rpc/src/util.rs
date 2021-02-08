// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use compiled_stdlib::shim::tmp_new_transaction_scripts;
use diem_crypto::HashValue;
use diem_json_rpc_types::views::{
    BytesView, MoveAbortExplanationView, ScriptView, TransactionDataView, VMStatusView,
};
use diem_types::{
    transaction::{Script, Transaction, TransactionArgument, TransactionPayload},
    vm_status::{AbortLocation, KeptVMStatus},
};
use move_core_types::language_storage::{StructTag, TypeTag};

/// Helper macros. Used to simplify adding new RpcHandler to Registry
/// `registry` - name of local registry variable
/// `name`  - name for the rpc method
/// `method` - method name of new rpc method
/// `required_num_args` - number of required method arguments
/// `opt_num_args` - number of optional method arguments
macro_rules! register_rpc_method {
    ($registry:expr, $name: expr, $method: expr, $required_num_args: expr, $opt_num_args: expr) => {
        $registry.insert(
            $name.to_string(),
            Box::new(move |service, request| {
                Box::pin(async move {
                    if request.params.len() < $required_num_args
                        || request.params.len() > $required_num_args + $opt_num_args
                    {
                        let expected = if $opt_num_args == 0 {
                            format!("{}", $required_num_args)
                        } else {
                            format!(
                                "{}..{}",
                                $required_num_args,
                                $required_num_args + $opt_num_args
                            )
                        };
                        anyhow::bail!(JsonRpcError::invalid_params_size(format!(
                            "wrong number of arguments (given {}, expected {})",
                            request.params.len(),
                            expected,
                        )));
                    }

                    fail_point!(format!("jsonrpc::method::{}", $name).as_str(), |_| {
                        Err(anyhow::format_err!("Injected error for method {} error", $name).into())
                    });
                    Ok(serde_json::to_value($method(service, request).await?)?)
                })
            }),
        );
    };
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
                _ => vec![],
            }
            .into();

            let script: ScriptView = match t.payload() {
                TransactionPayload::Script(s) => script_view_from_script(s),
                _ => ScriptView::unknown(),
            };

            TransactionDataView::UserTransaction {
                sender: t.sender().into(),
                signature_scheme: t.authenticator().scheme().to_string(),
                signature: t.authenticator().signature_bytes().into(),
                public_key: t.authenticator().public_key_bytes().into(),
                sequence_number: t.sequence_number(),
                chain_id: t.chain_id().id(),
                max_gas_amount: t.max_gas_amount(),
                gas_unit_price: t.gas_unit_price(),
                gas_currency: t.gas_currency_code().to_string(),
                expiration_timestamp_secs: t.expiration_timestamp_secs(),
                script_hash: script_hash.into(),
                script_bytes,
                script,
            }
        }
    }
}

pub fn script_view_from_script(script: &Script) -> ScriptView {
    let name = tmp_new_transaction_scripts::name_for_script(script.code())
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
            &script.args()[..]
        {
            view.receiver = Some(receiver.into());
            view.amount = Some(*amount);
            view.currency = Some(
                ty_args
                    .get(0)
                    .unwrap_or(&"unknown_currency".to_string())
                    .to_string(),
            );
            view.metadata = Some(BytesView::from(metadata));
            view.metadata_signature = Some(BytesView::from(metadata_signature));
        }
    }

    view
}
