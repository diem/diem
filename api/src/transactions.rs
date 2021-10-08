// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{context::Context, page::Page, param::TransactionIdParam};

use diem_api_types::{
    mime_types, Error, LedgerInfo, Response, Transaction, TransactionData, TransactionId,
};
use diem_types::{
    mempool_status::MempoolStatusCode,
    transaction::{SignedTransaction, TransactionInfo},
};

use anyhow::Result;
use warp::{
    http::{header::CONTENT_TYPE, StatusCode},
    reply, Filter, Rejection, Reply,
};

pub fn routes(context: Context) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    get_transaction(context.clone())
        .or(get_transactions(context.clone()))
        .or(post_bcs_transactions(context))
}

// GET /transactions/{txn-hash / version}
pub fn get_transaction(
    context: Context,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("transactions" / TransactionIdParam)
        .and(warp::get())
        .and(context.filter())
        .and_then(handle_get_transaction)
}

async fn handle_get_transaction(
    id: TransactionIdParam,
    context: Context,
) -> Result<impl Reply, Rejection> {
    Ok(Transactions::new(context)?
        .get_transaction(id.parse("transaction hash or version")?)
        .await?)
}

// GET /transactions?start={u64}&limit={u16}
pub fn get_transactions(
    context: Context,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("transactions")
        .and(warp::get())
        .and(warp::query::<Page>())
        .and(context.filter())
        .and_then(handle_get_transactions)
}

async fn handle_get_transactions(page: Page, context: Context) -> Result<impl Reply, Rejection> {
    Ok(Transactions::new(context)?.list(page)?)
}

// POST /transactions
pub fn post_bcs_transactions(
    context: Context,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("transactions")
        .and(warp::post())
        .and(warp::header::exact(
            CONTENT_TYPE.as_str(),
            mime_types::BCS_SIGNED_TRANSACTION,
        ))
        .and(warp::body::bytes())
        .and(context.filter())
        .and_then(handle_post_bcs_transactions)
}

async fn handle_post_bcs_transactions(
    body: bytes::Bytes,
    context: Context,
) -> Result<impl Reply, Rejection> {
    let txn = bcs::from_bytes(&body).map_err(|_| {
        Error::invalid_request_body("deserialize SignedTransaction BCS bytes failed".to_owned())
    })?;
    Ok(Transactions::new(context)?.create(txn).await?)
}

struct Transactions {
    ledger_info: LedgerInfo,
    context: Context,
}

impl Transactions {
    fn new(context: Context) -> Result<Self, Error> {
        let ledger_info = context.get_latest_ledger_info()?;
        Ok(Self {
            ledger_info,
            context,
        })
    }

    pub async fn create(self, txn: SignedTransaction) -> Result<impl Reply, Error> {
        let (mempool_status, vm_status_opt) = self.context.submit_transaction(txn.clone()).await?;
        match mempool_status.code {
            MempoolStatusCode::Accepted => {
                let converter = self.context.move_converter();
                let pending_txn = converter.try_into_pending_transaction(txn)?;
                let resp = Response::new(self.ledger_info, &pending_txn)?;
                Ok(reply::with_status(resp, StatusCode::ACCEPTED))
            }
            MempoolStatusCode::VmError => Err(Error::bad_request(format!(
                "invalid transaction: {}",
                vm_status_opt
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_else(|| "UNKNOWN".to_owned())
            ))),
            _ => Err(Error::bad_request(format!(
                "transaction is rejected: {}",
                mempool_status,
            ))),
        }
    }

    pub fn list(self, page: Page) -> Result<impl Reply, Error> {
        let ledger_version = self.ledger_info.version();
        let start_version = page.start(ledger_version)?;
        let limit = page.limit()?;

        let data = self
            .context
            .get_transactions(start_version, limit, ledger_version)?;

        let converter = self.context.move_converter();

        let txns: Vec<Transaction> = data
            .into_iter()
            .map(|t| converter.try_into_onchain_transaction(t))
            .collect::<Result<_>>()?;
        Response::new(self.ledger_info, &txns)
    }

    pub async fn get_transaction(self, id: TransactionId) -> Result<impl Reply, Error> {
        let txn_data = match id.clone() {
            TransactionId::Hash(hash) => self.get_by_hash(hash.into()).await?,
            TransactionId::Version(version) => self.get_by_version(version)?,
        }
        .ok_or_else(|| self.transaction_not_found(id))?;

        let converter = self.context.move_converter();
        let ret = converter.try_into_transaction(txn_data)?;

        Response::new(self.ledger_info, &ret)
    }

    fn transaction_not_found(&self, id: TransactionId) -> Error {
        Error::not_found("transaction", id, self.ledger_info.version())
    }

    fn get_by_version(&self, version: u64) -> Result<Option<TransactionData<TransactionInfo>>> {
        if version > self.ledger_info.version() {
            return Ok(None);
        }
        Ok(Some(
            self.context
                .get_transaction_by_version(version, self.ledger_info.version())?
                .into(),
        ))
    }

    // This function looks for the transaction by hash in database and then mempool,
    // because the period a transaction stay in the mempool is likely short.
    // Although the mempool get transation is async, but looking up txn in database is a sync call,
    // thus we keep it simple and call them in sequence.
    async fn get_by_hash(
        &self,
        hash: diem_crypto::HashValue,
    ) -> Result<Option<TransactionData<TransactionInfo>>> {
        let from_db = self
            .context
            .get_transaction_by_hash(hash, self.ledger_info.version())?;
        Ok(match from_db {
            None => self
                .context
                .get_pending_transaction_by_hash(hash)
                .await?
                .map(|t| t.into()),
            _ => from_db.map(|t| t.into()),
        })
    }
}

#[cfg(any(test))]
mod tests {
    use crate::test_utils::{assert_json, find_value, new_test_context};

    use diem_crypto::{
        hash::CryptoHash,
        multi_ed25519::{MultiEd25519PrivateKey, MultiEd25519PublicKey},
        SigningKey, Uniform,
    };
    use diem_sdk::{client::SignedTransaction, transaction_builder::Currency};
    use diem_types::{
        access_path::{AccessPath, Path},
        account_address::AccountAddress,
        transaction::{
            authenticator::{AuthenticationKey, TransactionAuthenticator},
            ChangeSet, Transaction, TransactionInfoTrait,
        },
        write_set::{WriteOp, WriteSetMut},
    };

    use move_core_types::{
        identifier::Identifier,
        language_storage::{ModuleId, StructTag},
    };
    use serde_json::json;

    #[tokio::test]
    async fn test_get_transactions_output_genesis_transaction() {
        let context = new_test_context();
        let ledger_info = context.get_latest_ledger_info();
        let txns = context
            .context
            .get_transactions(0, 1, ledger_info.version())
            .unwrap();

        let resp = context.get("/transactions").await;
        assert_eq!(1, resp.as_array().unwrap().len());
        let txn = &resp[0];
        assert_eq!(txn["type"], "genesis_transaction");
        assert_eq!(txn["version"], "0");

        let info = txns[0].info.clone();
        assert_eq!(txn["hash"], info.transaction_hash().to_hex_literal());
        assert_eq!(
            txn["state_root_hash"],
            info.state_root_hash().to_hex_literal()
        );
        assert_eq!(
            txn["event_root_hash"],
            info.event_root_hash().to_hex_literal()
        );

        let chain_id = find_value(&txn["payload"]["changes"], |val| {
            val["type"] == "write_module" && val["data"]["abi"]["name"] == "ChainId"
        });
        let bytecode = chain_id["data"]["bytecode"].clone();
        assert!(bytecode.as_str().unwrap().starts_with("0x"));
        assert_json(
            chain_id,
            json!({
                "type": "write_module",
                "address": "0x1",
                "data": {
                    "bytecode": bytecode.as_str().unwrap(),
                    "abi": {
                        "address": "0x1",
                        "name": "ChainId",
                        "friends": [],
                        "exposed_functions": [
                            {
                                "name": "get",
                                "visibility": "public",
                                "generic_type_params": [],
                                "params": [],
                                "return": [
                                    {
                                        "type": "u8"
                                    }
                                ]
                            },
                            {
                                "name": "initialize",
                                "visibility": "public",
                                "generic_type_params": [],
                                "params": [
                                    {
                                        "type": "reference",
                                        "mutable": false,
                                        "to": {
                                            "type": "signer"
                                        }
                                    },
                                    {
                                        "type": "u8"
                                    }
                                ],
                                "return": []
                            }
                        ],
                        "structs": [
                            {
                                "name": "ChainId",
                                "is_native": false,
                                "abilities": [
                                    "key"
                                ],
                                "generic_type_params": [],
                                "fields": [
                                    {
                                        "name": "id",
                                        "type": {
                                            "type": "u8"
                                        }
                                    }
                                ]
                            }
                        ]
                    }
                }
            }),
        );

        let chain_id = find_value(&txn["payload"]["changes"], |val| {
            val["type"] == "write_resource"
                && val["address"] == "0xdd"
                && val["data"]["type"]["name"] == "RoleId"
        });
        assert_json(
            chain_id,
            json!({
                "type": "write_resource",
                "address": "0xdd",
                "data": {
                    "type": {
                        "type": "struct",
                        "address": "0x1",
                        "module": "Roles",
                        "name": "RoleId",
                        "generic_type_params": []
                    },
                    "value": {
                        "role_id": "2"
                    }
                }
            }),
        );

        let first_event = txn["events"][0].clone();
        // transaction events are same with events from payload
        assert_json(first_event.clone(), txn["payload"]["events"][0].clone());
        assert_json(
            first_event,
            json!({
                "key": "0x00000000000000000000000000000000000000000a550c18",
                "sequence_number": "0",
                "type": {
                    "type": "struct",
                    "address": "0x1",
                    "module": "DiemAccount",
                    "name": "CreateAccountEvent",
                    "generic_type_params": []
                },
                "data": {
                    "created": "0xa550c18",
                    "role_id": "0"
                }
            }),
        );
    }

    #[tokio::test]
    async fn test_get_transactions_with_start_version_is_too_large() {
        let context = new_test_context();
        let ledger_version = context.get_latest_ledger_info().version();
        let resp = context
            .expect_status_code(404)
            .get("/transactions?start=1000000&limit=10")
            .await;
        assert_json(
            resp,
            json!({
              "code": 404,
              "message": "transaction not found by version(1000000)",
              "diem_ledger_version": ledger_version.to_string()
            }),
        );
    }

    #[tokio::test]
    async fn test_get_transactions_with_invalid_start_version_param() {
        let context = new_test_context();
        let resp = context
            .expect_status_code(400)
            .get("/transactions?start=hello")
            .await;
        assert_json(
            resp,
            json!({
              "code": 400,
              "message": "invalid parameter start: hello"
            }),
        );
    }

    #[tokio::test]
    async fn test_get_transactions_with_invalid_limit_param() {
        let context = new_test_context();
        let resp = context
            .expect_status_code(400)
            .get("/transactions?limit=hello")
            .await;
        assert_json(
            resp,
            json!({
              "code": 400,
              "message": "invalid parameter limit: hello"
            }),
        );
    }

    #[tokio::test]
    async fn test_get_transactions_param_limit_exceeds_limit() {
        let context = new_test_context();
        let resp = context
            .expect_status_code(400)
            .get("/transactions?limit=2000")
            .await;
        assert_json(
            resp,
            json!({
              "code": 400,
              "message": "invalid parameter limit: 2000, exceed limit 1000"
            }),
        );
    }

    #[tokio::test]
    async fn test_get_transactions_output_user_transaction_with_script_function_payload() {
        let mut context = new_test_context();
        let account = context.gen_account();
        let txn = context.create_parent_vasp(&account);
        context.commit_block(&vec![txn.clone()]);

        let txns = context.get("/transactions?start=1").await;
        assert_eq!(2, txns.as_array().unwrap().len());

        let expected_txns = context.get_transactions(1, 2);
        assert_eq!(2, expected_txns.len());

        let metadata = expected_txns[0].info.clone();

        let metadata_txn = match &expected_txns[0].transaction {
            Transaction::BlockMetadata(txn) => txn.clone(),
            _ => panic!("unexpected transaction: {:?}", expected_txns[0].transaction),
        };
        assert_json(
            txns[0].clone(),
            json!(
            {
                "type": "block_metadata_transaction",
                "version": "1",
                "hash": metadata.transaction_hash().to_hex_literal(),
                "state_root_hash": metadata.state_root_hash().to_hex_literal(),
                "event_root_hash": metadata.event_root_hash().to_hex_literal(),
                "gas_used": metadata.gas_used().to_string(),
                "success": true,
                "id": metadata_txn.id().to_hex_literal(),
                "round": "1",
                "previous_block_votes": [],
                "proposer": context.validator_owner.to_hex_literal(),
                "timestamp": metadata_txn.timestamp_usec().to_string(),
            }),
        );

        let user_txn_info = expected_txns[1].info.clone();
        let (public_key, sig) = match txn.authenticator() {
            TransactionAuthenticator::Ed25519 {
                public_key,
                signature,
            } => (public_key, signature),
            _ => panic!(
                "expecting TransactionAuthenticator::Ed25519, but got: {:?}",
                txn.authenticator()
            ),
        };
        assert_json(
            txns[1].clone(),
            json!({
                "type": "user_transaction",
                "version": "2",
                "hash": user_txn_info.transaction_hash().to_hex_literal(),
                "state_root_hash": user_txn_info.state_root_hash().to_hex_literal(),
                "event_root_hash": user_txn_info.event_root_hash().to_hex_literal(),
                "gas_used": user_txn_info.gas_used().to_string(),
                "success": true,
                "sender": "0xb1e55ed",
                "sequence_number": "0",
                "max_gas_amount": "1000000",
                "gas_unit_price": "0",
                "gas_currency_code": "XUS",
                "expiration_timestamp_secs": txn.expiration_timestamp_secs().to_string(),
                "events": [
                    {
                        "key": "0x00000000000000000000000000000000000000000a550c18",
                        "sequence_number": "5",
                        "type": {
                            "type": "struct",
                            "address": "0x1",
                            "module": "DiemAccount",
                            "name": "CreateAccountEvent",
                            "generic_type_params": []
                        },
                        "data": {
                            "created": account.address().to_hex_literal(),
                            "role_id": "5"
                        }
                    }
                ],
                "payload": {
                    "type": "script_function_payload",
                    "module": {
                        "address": "0x1",
                        "name": "AccountCreationScripts"
                    },
                    "function": "create_parent_vasp_account",
                    "type_arguments": [
                        {
                            "type": "struct",
                            "address": "0x1",
                            "module": "XUS",
                            "name": "XUS",
                            "generic_type_params": []
                        }
                    ],
                    "arguments": [
                        "0",
                        account.address().to_hex_literal(),
                        format!("0x{}", hex::encode(account.authentication_key().prefix())),
                        format!("0x{}", hex::encode("vasp".as_bytes())),
                        true
                    ]
                },
                "signature": {
                    "type": "ed25519_signature",
                    "public_key": format!("0x{}", hex::encode(public_key.unvalidated().to_bytes())),
                    "signature": format!("0x{}", hex::encode(sig.to_bytes())),
                },
            }),
        )
    }

    #[tokio::test]
    async fn test_get_transactions_output_user_transaction_with_script_payload() {
        let context = new_test_context();
        let new_key = "717d1d400311ff8797c2441ea9c2d2da1120ce38f66afb079c2bad0919d93a09"
            .parse()
            .unwrap();

        let mut tc_account = context.tc_account();
        let txn = tc_account.sign_with_transaction_builder(
            context
                .transaction_factory()
                .rotate_authentication_key_by_script(new_key),
        );
        context.commit_block(&vec![txn.clone()]);

        let txns = context.get("/transactions?start=2").await;
        assert_eq!(1, txns.as_array().unwrap().len());

        let expected_txns = context.get_transactions(2, 1);
        assert_eq!(1, expected_txns.len());

        assert_json(
            txns[0]["payload"].clone(),
            json!({
                "type": "script_payload",
                "code": {
                    "bytecode": "0xa11ceb0b010000000601000202020403060f05151207277c08a3011000000001010000020001000003010200000403020001060c01080000020608000a0202060c0a020b4469656d4163636f756e74154b6579526f746174696f6e4361706162696c6974791f657874726163745f6b65795f726f746174696f6e5f6361706162696c6974791f726573746f72655f6b65795f726f746174696f6e5f6361706162696c69747919726f746174655f61757468656e7469636174696f6e5f6b657900000000000000000000000000000001000401090b0011000c020e020b0111020b02110102",
                    "abi": {
                        "name": "main",
                        "visibility": "script",
                        "generic_type_params": [],
                        "params": [
                            {
                                "type": "reference",
                                "mutable": false,
                                "to": {
                                    "type": "signer"
                                }
                            },
                            {
                                "type": "vector",
                                "items": {
                                    "type": "u8"
                                }
                            }
                        ],
                        "return": []
                    }
                },
                "type_arguments": [],
                "arguments": [
                    "0x717d1d400311ff8797c2441ea9c2d2da1120ce38f66afb079c2bad0919d93a09"
                ]
            }),
        )
    }

    #[tokio::test]
    async fn test_get_transactions_output_user_transaction_with_module_payload() {
        let context = new_test_context();
        let code = "a11ceb0b0300000006010002030205050703070a0c0816100c260900000001000100000102084d794d6f64756c650269640000000000000000000000000b1e55ed00010000000231010200";
        let mut tc_account = context.tc_account();
        let txn = tc_account.sign_with_transaction_builder(
            context
                .transaction_factory()
                .module(hex::decode(code).unwrap()),
        );
        context.commit_block(&vec![txn.clone()]);

        let txns = context.get("/transactions?start=2").await;
        assert_eq!(1, txns.as_array().unwrap().len());

        let expected_txns = context.get_transactions(2, 1);
        assert_eq!(1, expected_txns.len());

        assert_json(
            txns[0]["payload"].clone(),
            json!({
                "type": "module_payload",
                "bytecode": format!("0x{}", code),
                "abi": {
                    "address": "0xb1e55ed",
                    "name": "MyModule",
                    "friends": [],
                    "exposed_functions": [
                        {
                            "name": "id",
                            "visibility": "public",
                            "generic_type_params": [],
                            "params": [],
                            "return": [
                                {
                                    "type": "u8"
                                }
                            ]
                        }
                    ],
                    "structs": []
                }
            }),
        )
    }

    #[tokio::test]
    async fn test_get_transactions_output_user_transaction_with_write_set_payload() {
        let context = new_test_context();
        let mut root_account = context.root_account();
        let code_address = AccountAddress::from_hex_literal("0x1").unwrap();
        let txn = root_account.sign_with_transaction_builder(
            context.transaction_factory().change_set(ChangeSet::new(
                WriteSetMut::new(vec![
                    (
                        AccessPath::new(
                            code_address,
                            bcs::to_bytes(&Path::Code(ModuleId::new(
                                code_address,
                                Identifier::new("AccountAdministrationScripts").unwrap(),
                            )))
                            .unwrap(),
                        ),
                        WriteOp::Deletion,
                    ),
                    (
                        AccessPath::new(
                            context.tc_account().address(),
                            bcs::to_bytes(&Path::Resource(StructTag {
                                address: code_address,
                                module: Identifier::new("AccountFreezing").unwrap(),
                                name: Identifier::new("FreezingBit").unwrap(),
                                type_params: vec![],
                            }))
                            .unwrap(),
                        ),
                        WriteOp::Deletion,
                    ),
                ])
                .freeze()
                .unwrap(),
                vec![],
            )),
        );
        context.commit_block(&vec![txn.clone()]);

        let txns = context.get("/transactions?start=2").await;
        assert_eq!(1, txns.as_array().unwrap().len());

        assert_json(
            txns[0]["payload"].clone(),
            json!({
                "type": "direct_write_set",
                "changes": [
                    {
                        "type": "delete_module",
                        "address": "0x1",
                        "module": {
                            "address": "0x1",
                            "name": "AccountAdministrationScripts"
                        }
                    },
                    {
                        "type": "delete_resource",
                        "address": "0xb1e55ed",
                        "resource": {
                            "type": "struct",
                            "address": "0x1",
                            "module": "AccountFreezing",
                            "name": "FreezingBit",
                            "generic_type_params": []
                        }
                    }
                ],
                "events": []
            }),
        )
    }

    #[tokio::test]
    async fn test_post_bcs_format_transaction() {
        let mut context = new_test_context();
        let account = context.gen_account();
        let txn = context.create_parent_vasp(&account);
        let body = bcs::to_bytes(&txn).unwrap();
        let resp = context
            .expect_status_code(202)
            .post_bcs_txn("/transactions", body)
            .await;
        let expiration_timestamp = txn.expiration_timestamp_secs();
        let (public_key, sig) = match txn.authenticator() {
            TransactionAuthenticator::Ed25519 {
                public_key,
                signature,
            } => (public_key, signature),
            _ => panic!(
                "expecting TransactionAuthenticator::Ed25519, but got: {:?}",
                txn.authenticator()
            ),
        };
        let hash = Transaction::UserTransaction(txn).hash();
        assert_json(
            resp,
            json!({
                "type": "pending_transaction",
                "hash": hash.to_hex_literal(),
                "sender": "0xb1e55ed",
                "sequence_number": "0",
                "max_gas_amount": "1000000",
                "gas_unit_price": "0",
                "gas_currency_code": "XUS",
                "expiration_timestamp_secs": expiration_timestamp.to_string(),
                "payload": {
                    "type": "script_function_payload",
                    "module": {
                        "address": "0x1",
                        "name": "AccountCreationScripts"
                    },
                    "function": "create_parent_vasp_account",
                    "type_arguments": [
                        {
                            "type": "struct",
                            "address": "0x1",
                            "module": "XUS",
                            "name": "XUS",
                            "generic_type_params": []
                        }
                    ],
                    "arguments": [
                        "0",
                        account.address().to_hex_literal(),
                        format!("0x{}", hex::encode(account.authentication_key().prefix())),
                        format!("0x{}", hex::encode("vasp".as_bytes())),
                        true
                    ]
                },
                "signature": {
                    "type": "ed25519_signature",
                    "public_key": format!("0x{}", hex::encode(public_key.unvalidated().to_bytes())),
                    "signature": format!("0x{}", hex::encode(sig.to_bytes())),
                },
            }),
        );
    }

    #[tokio::test]
    async fn test_post_invalid_bcs_format_transaction() {
        let context = new_test_context();

        let resp = context
            .expect_status_code(400)
            .post_bcs_txn("/transactions", bcs::to_bytes("invalid data").unwrap())
            .await;
        assert_json(
            resp,
            json!({
              "code": 400,
              "message": "invalid request body: deserialize SignedTransaction BCS bytes failed"
            }),
        );
    }

    #[tokio::test]
    async fn test_post_invalid_signature_transaction() {
        let mut context = new_test_context();
        let txn = context.create_invalid_signature_transaction();
        let body = bcs::to_bytes(&txn).unwrap();
        let resp = context
            .expect_status_code(400)
            .post_bcs_txn("/transactions", &body)
            .await;
        assert_json(
            resp,
            json!({
              "code": 400,
              "message": "invalid transaction: INVALID_SIGNATURE"
            }),
        );
    }

    #[tokio::test]
    async fn test_post_transaction_rejected_by_mempool() {
        let mut context = new_test_context();
        let account1 = context.gen_account();
        let account2 = context.gen_account();
        let txn1 = context.create_parent_vasp(&account1);
        let txn2 = context.create_parent_vasp(&account2);

        context
            .expect_status_code(202)
            .post_bcs_txn("/transactions", &bcs::to_bytes(&txn1).unwrap())
            .await;

        let resp = context
            .expect_status_code(400)
            .post_bcs_txn("/transactions", &bcs::to_bytes(&txn2).unwrap())
            .await;
        assert_json(
            resp,
            json!({
              "code": 400,
              "message": "transaction is rejected: InvalidUpdate - Failed to update gas price to 0"
            }),
        );
    }

    #[tokio::test]
    async fn test_multi_agent_signed_transaction() {
        let mut context = new_test_context();
        let account = context.gen_account();
        let factory = context.transaction_factory();
        let mut tc_account = context.tc_account();
        let secondary = context.root_account();
        let txn = tc_account.sign_multi_agent_with_transaction_builder(
            vec![&secondary],
            factory.create_parent_vasp_account(
                Currency::XUS,
                0,
                account.authentication_key(),
                "vasp",
                true,
            ),
        );

        let body = bcs::to_bytes(&txn).unwrap();
        let resp = context
            .expect_status_code(202)
            .post_bcs_txn("/transactions", body)
            .await;

        let (sender, secondary_signers) = match txn.authenticator() {
            TransactionAuthenticator::MultiAgent {
                sender,
                secondary_signer_addresses: _,
                secondary_signers,
            } => (sender, secondary_signers),
            _ => panic!(
                "expecting TransactionAuthenticator::MultiAgent, but got: {:?}",
                txn.authenticator()
            ),
        };
        assert_json(
            resp["signature"].clone(),
            json!({
                "type": "multi_agent_signature",
                "sender": {
                    "type": "ed25519_signature",
                    "public_key": format!("0x{}", hex::encode(sender.public_key_bytes())),
                    "signature": format!("0x{}", hex::encode(sender.signature_bytes())),
                },
                "secondary_signer_addresses": [
                    secondary.address().to_hex_literal(),
                ],
                "secondary_signers": [
                    {
                        "type": "ed25519_signature",
                        "public_key": format!("0x{}",hex::encode(secondary_signers[0].public_key_bytes())),
                        "signature": format!("0x{}", hex::encode(secondary_signers[0].signature_bytes())),
                    }
                ]
            }),
        );
    }

    #[tokio::test]
    async fn test_multi_ed25519_signed_transaction() {
        let context = new_test_context();

        let private_key = MultiEd25519PrivateKey::generate_for_testing();
        let public_key = MultiEd25519PublicKey::from(&private_key);
        let auth_key = AuthenticationKey::multi_ed25519(&public_key);

        let factory = context.transaction_factory();
        let mut tc_account = context.tc_account();
        let create_account_txn = tc_account.sign_with_transaction_builder(
            factory.create_parent_vasp_account(Currency::XUS, 0, auth_key, "vasp", true),
        );
        context.commit_block(&vec![create_account_txn]);

        let raw_txn = factory
            .create_recovery_address()
            .sender(auth_key.derived_address())
            .sequence_number(0)
            .build();

        let signature = private_key.sign(&raw_txn);
        let txn = SignedTransaction::new_multisig(raw_txn, public_key, signature.clone());

        let body = bcs::to_bytes(&txn).unwrap();
        let resp = context
            .expect_status_code(202)
            .post_bcs_txn("/transactions", body)
            .await;

        assert_json(
            resp["signature"].clone(),
            json!({
                "type": "multi_ed25519_signature",
                "signatures": [
                    {
                        "public_key": "0x9e4208caddd825f71957c9b12dbfbd13a23fb0ea23eb398fd7e1f418b51f8fbc",
                        "signature": format!("0x{}", hex::encode(signature.signatures()[0].to_bytes())),
                    },
                    {
                        "public_key": "0x4708a77bb9285ce3745ffdd48c51980326b625488209803228ff623f3768c64e",
                        "signature": format!("0x{}", hex::encode(signature.signatures()[1].to_bytes())),
                    },
                    {
                        "public_key": "0x852b13cd7a89b0c223d74504705e84c745d32261244ed233ef0285637a1dece0",
                        "signature": format!("0x{}", hex::encode(signature.signatures()[2].to_bytes())),
                    }
                ],
                "threshold": 3,
                "bitmap": "0xe0000000"
            }),
        );
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash() {
        let mut context = new_test_context();
        let account = context.gen_account();
        let txn = context.create_parent_vasp(&account);
        context.commit_block(&vec![txn.clone()]);

        let txns = context.get("/transactions?start=2").await;
        assert_eq!(1, txns.as_array().unwrap().len());

        let resp = context
            .get(&format!(
                "/transactions/{}",
                txns[0]["hash"].as_str().unwrap()
            ))
            .await;
        assert_json(resp, txns[0].clone())
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash_not_found() {
        let context = new_test_context();

        let resp = context
            .expect_status_code(404)
            .get("/transactions/0xdadfeddcca7cb6396c735e9094c76c6e4e9cb3e3ef814730693aed59bd87b31d")
            .await;
        assert_json(
            resp,
            json!({
                "code": 404,
                "message": "transaction not found by hash(0xdadfeddcca7cb6396c735e9094c76c6e4e9cb3e3ef814730693aed59bd87b31d)",
                "diem_ledger_version": "0"
            }),
        )
    }

    #[tokio::test]
    async fn test_get_transaction_by_invalid_hash() {
        let context = new_test_context();

        let resp = context
            .expect_status_code(400)
            .get(&format!("/transactions/{}", "0x1",))
            .await;
        assert_json(
            resp,
            json!({
                "code": 400,
                "message": "invalid parameter transaction hash or version: 0x1"
            }),
        )
    }

    #[tokio::test]
    async fn test_get_transaction_by_version_not_found() {
        let context = new_test_context();

        let resp = context
            .expect_status_code(404)
            .get("/transactions/10000")
            .await;
        assert_json(
            resp,
            json!({
                "code": 404,
                "message": "transaction not found by version(10000)",
                "diem_ledger_version": "0"
            }),
        )
    }

    #[tokio::test]
    async fn test_get_transaction_by_version() {
        let mut context = new_test_context();
        let account = context.gen_account();
        let txn = context.create_parent_vasp(&account);
        context.commit_block(&vec![txn.clone()]);

        let txns = context.get("/transactions?start=2").await;
        assert_eq!(1, txns.as_array().unwrap().len());

        let resp = context.get("/transactions/2").await;
        assert_json(resp, txns[0].clone())
    }

    #[tokio::test]
    async fn test_get_pending_transaction_by_hash() {
        let mut context = new_test_context();
        let account = context.gen_account();
        let txn = context.create_parent_vasp(&account);
        let body = bcs::to_bytes(&txn).unwrap();
        let pending_txn = context
            .expect_status_code(202)
            .post_bcs_txn("/transactions", body)
            .await;

        let txn_hash = pending_txn["hash"].as_str().unwrap();

        let txn = context.get(&format!("/transactions/{}", txn_hash)).await;
        assert_json(txn, pending_txn);

        let not_found = context
            .expect_status_code(404)
            .get("/transactions/0xdadfeddcca7cb6396c735e9094c76c6e4e9cb3e3ef814730693aed59bd87b31d")
            .await;
        assert_json(
            not_found,
            json!({
                "code": 404,
                "message": "transaction not found by hash(0xdadfeddcca7cb6396c735e9094c76c6e4e9cb3e3ef814730693aed59bd87b31d)",
                "diem_ledger_version": "0"
            }),
        )
    }
}
