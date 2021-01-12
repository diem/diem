// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

use compiled_stdlib::transaction_scripts::StdlibScript;
use diem_crypto::HashValue;
use diem_types::transaction::{Transaction, TransactionPayload, WriteSetPayload};

use crate::ScriptExecPack;

pub struct AnalyzerE2E {
    diem_scripts: BTreeMap<HashValue, StdlibScript>,
    _output_dir: PathBuf,
}

impl AnalyzerE2E {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            diem_scripts: StdlibScript::all()
                .into_iter()
                .map(|script| (script.hash(), script))
                .collect(),
            _output_dir: output_dir,
        }
    }

    fn follow_one_trace(
        &self,
        dir: &Path,
        exec_packs: &mut BTreeMap<HashValue, HashSet<ScriptExecPack>>,
    ) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let item = entry?.path();
            let file = File::open(item)?;
            let txn: Transaction = serde_json::from_reader(BufReader::new(file))?;

            if let Transaction::UserTransaction(signed_txn) = txn {
                let sender_script_opt = match signed_txn.payload() {
                    TransactionPayload::Script(script) => Some((signed_txn.sender(), script)),
                    TransactionPayload::Module(_) => None,
                    TransactionPayload::WriteSet(WriteSetPayload::Direct(_)) => None,
                    TransactionPayload::WriteSet(WriteSetPayload::Script {
                        execute_as,
                        script,
                    }) => Some((*execute_as, script)),
                };

                if let Some((sender, script)) = sender_script_opt {
                    let hash = HashValue::sha3_256_of(script.code());
                    if self.diem_scripts.get(&hash).is_some() {
                        let pack = ScriptExecPack {
                            sender,
                            ty_args: script.ty_args().to_vec(),
                            args: script.args().to_vec(),
                        };
                        exec_packs
                            .entry(hash)
                            .or_insert_with(HashSet::new)
                            .insert(pack);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn run_on_trace_dir<P: AsRef<Path>>(&self, trace_dir: P) -> Result<()> {
        let mut exec_packs = BTreeMap::new();
        for entry in fs::read_dir(trace_dir)? {
            let item = entry?.path();
            self.follow_one_trace(&item, &mut exec_packs)?
        }
        for (hash, packs) in exec_packs.into_iter() {
            let diem_script = self.diem_scripts.get(&hash).unwrap();
            println!(
                "Diem script {} is invoked {} times",
                diem_script.name(),
                packs.len()
            );
            for pack in packs {
                println!("\t{}", pack);
            }
        }
        Ok(())
    }
}
