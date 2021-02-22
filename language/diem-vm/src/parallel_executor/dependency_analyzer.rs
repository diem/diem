// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_cache::StateViewCache,
    diem_transaction_executor::{preprocess_transaction, PreprocessedTransaction},
    logging::AdapterLogSchema,
    DiemVM,
};
use diem_state_view::StateView;
use diem_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    transaction::{Transaction, TransactionArgument, TransactionPayload},
};
use move_core_types::vm_status::{KeptVMStatus, StatusCode, VMStatus};
use std::collections::{HashMap, HashSet};

// Structure that holds infered read/write sets

#[derive(Debug, Clone)]
enum ScriptReadWriteSetVar {
    Const,
    Param(usize),
}

#[derive(Debug)]
struct ScriptReadWriteSet {
    reads: Vec<(ScriptReadWriteSetVar, AccessPath)>,
    writes: Vec<(ScriptReadWriteSetVar, AccessPath)>,
}

impl ScriptReadWriteSet {
    // Given a set of address parameters, by convention [Sender, Address, Address, ...], and some read and write
    // access paths, it infers which are static and which dynamic, stores the structure to allow inference about others.
    pub fn new(
        params: &[AccountAddress],
        reads: Vec<AccessPath>,
        writes: Vec<AccessPath>,
    ) -> ScriptReadWriteSet {
        ScriptReadWriteSet {
            reads: reads
                .into_iter()
                .map(|path| {
                    let var = match params.iter().position(|&x| x == path.address) {
                        None => ScriptReadWriteSetVar::Const,
                        Some(i) => ScriptReadWriteSetVar::Param(i),
                    };
                    (var, path)
                })
                .collect(),
            writes: writes
                .into_iter()
                .map(|path| {
                    let var = match params.iter().position(|&x| x == path.address) {
                        None => ScriptReadWriteSetVar::Const,
                        Some(i) => ScriptReadWriteSetVar::Param(i),
                    };
                    (var, path)
                })
                .collect(),
        }
    }

    // Return the read access paths specialized for these parameters
    // TODO: return a result in case the params are not long enough.
    pub fn reads<'a>(&'a self, params: &'a TransactionParameters) -> ScriptReadWriteSetVarIter {
        return ScriptReadWriteSetVarIter::new(&self.reads, params);
    }

    // Return the write access paths specialized for these parameters
    // TODO: return a result in case the params are not long enough.
    pub fn writes<'a>(
        &'a self,
        params: &'a TransactionParameters,
    ) -> ScriptReadWriteSetVarIter<'a> {
        return ScriptReadWriteSetVarIter::new(&self.writes, params);
    }
}

#[derive(Debug)]
pub(crate) struct TransactionParameters(Vec<AccountAddress>);

impl TransactionParameters {
    pub fn new_from(txn: &PreprocessedTransaction) -> Self {
        if let PreprocessedTransaction::UserTransaction(user_txn) = txn {
            match user_txn.payload() {
                TransactionPayload::Script(script) => {
                    // If the transaction is not known, then execute it to infer its read/write logic.
                    let mut params = Vec::with_capacity(5);
                    params.push(user_txn.sender());
                    for arg in script.args() {
                        if let TransactionArgument::Address(address) = arg {
                            params.push(address.clone());
                        }
                    }
                    Self(params)
                }
                _ => {
                    unimplemented!("FIXME")
                }
            }
        } else {
            unimplemented!("FIXME")
        }
    }

    pub fn args(&self) -> &[AccountAddress] {
        &self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ScriptReadWriteSetVarIter<'a> {
    // A link to the array we iterate over
    array: &'a [(ScriptReadWriteSetVar, AccessPath)],
    // The parameters we use to popular the read-write set
    params: &'a TransactionParameters,
    // the position we are in the array.
    seq: usize,
}

impl<'a> ScriptReadWriteSetVarIter<'a> {
    fn new(
        array: &'a Vec<(ScriptReadWriteSetVar, AccessPath)>,
        params: &'a TransactionParameters,
    ) -> ScriptReadWriteSetVarIter<'a> {
        ScriptReadWriteSetVarIter {
            array,
            params,
            seq: 0,
        }
    }
}

impl<'a> Iterator for ScriptReadWriteSetVarIter<'a> {
    // we will be counting with usize
    type Item = AccessPath;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        if self.seq < self.array.len() {
            let (v, p) = &self.array[self.seq];
            let current_item = match v {
                ScriptReadWriteSetVar::Const => p.clone(),
                ScriptReadWriteSetVar::Param(i) => {
                    let mut p = p.clone();
                    p.address = self.params.args()[*i];
                    p
                }
            };
            self.seq += 1;
            Some(current_item)
        } else {
            None
        }
    }
}

pub(crate) struct DependencyAnalyzer {
    script_map: HashMap<Vec<u8>, ScriptReadWriteSet>,
}

impl DependencyAnalyzer {
    pub fn new_from_transactions(
        transactions: &[PreprocessedTransaction],
        data_cache: &StateViewCache,
    ) -> Result<Self, VMStatus> {
        // STARTS -- DEPENDENCY INFERENCE HACK / Replace with static analysis of write set
        let mut read_write_infer = HashMap::<Vec<u8>, ScriptReadWriteSet>::new();
        let diem_vm = DiemVM::new(data_cache);

        for txn in transactions.iter() {
            if let PreprocessedTransaction::UserTransaction(user_txn) = txn {
                if let TransactionPayload::Script(script) = user_txn.payload() {
                    // If the transaction is not known, then execute it to infer its read/write logic.
                    if !read_write_infer.contains_key(script.code()) {
                        let xref = &*data_cache;
                        let local_state_view_cache = StateViewCache::new_recorder(xref);
                        let log_context = AdapterLogSchema::new(xref.id(), 0);
                        // Execute the transaction
                        if let Ok((status, output, _)) = diem_vm.execute_single_transaction(
                            &txn,
                            &local_state_view_cache,
                            &log_context,
                        ) {
                            match status.keep_or_discard() {
                                Ok(KeptVMStatus::Executed) => (),
                                _ => continue,
                            };
                            // Record the read-set
                            let read_set = local_state_view_cache.read_set();

                            // Create a params list
                            let params = TransactionParameters::new_from(txn);

                            let mut reads = Vec::new();
                            let mut writes = Vec::new();
                            let write_set: HashSet<AccessPath> =
                                output.write_set().iter().map(|(k, _)| k).cloned().collect();

                            for path in read_set {
                                if write_set.contains(&path) {
                                    reads.push(path.clone());
                                    writes.push(path.clone());
                                // println!("  -W {}", path);
                                } else {
                                    reads.push(path.clone());
                                    // println!("  -R {}", path);
                                }
                            }

                            read_write_infer.insert(
                                script.code().to_vec(),
                                ScriptReadWriteSet::new(params.args(), reads, writes),
                            );
                        } else {
                            panic!("NO LOGIC TO INFER READ/WRITE SET");
                        }
                    }
                } else {
                    unimplemented!();
                }
            } else {
                unimplemented!();
            }
        }

        // ENDS
        Ok(Self {
            script_map: read_write_infer,
        })
    }

    pub fn get_inferred_read_write_set<'a>(
        &'a self,
        txn: &PreprocessedTransaction,
        params: &'a TransactionParameters,
    ) -> Result<
        (
            impl Iterator<Item = AccessPath> + 'a + Copy,
            impl Iterator<Item = AccessPath> + 'a + Copy,
        ),
        VMStatus,
    > {
        if let PreprocessedTransaction::UserTransaction(user_txn) = txn {
            if let TransactionPayload::Script(script) = user_txn.payload() {
                let deps = match self.script_map.get(script.code()) {
                    Some(deps) => deps,
                    None => return Err(VMStatus::Error(StatusCode::UNKNOWN_VALIDATION_STATUS)),
                };

                return Ok((deps.reads(params), deps.writes(params)));
            }
        }
        Err(VMStatus::Error(StatusCode::UNKNOWN_VALIDATION_STATUS))
    }
}
