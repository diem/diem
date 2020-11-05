// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::File,
    io::Read,
    iter::FromIterator,
};

#[derive(Debug, Default)]
pub struct SimpleEnv {
    scripts: HashSet<String>,
    modules: HashSet<String>,
    functions: HashSet<String>,
    callers: HashMap<String, HashSet<String>>,
    callees: HashMap<String, HashSet<String>>,
    public_functions: HashSet<String>,
    true_public_functions: HashSet<String>,
    private_functions: HashSet<String>,
    native_functions: HashSet<String>,
}

impl SimpleEnv {
    pub fn from_json(filename: &str) -> SimpleEnv {
        #[derive(Debug, Deserialize)]
        struct FunctionEntry {
            name: String,
            calls: Vec<String>,
            called_by: Vec<String>,
            is_public: bool,
            is_called_by_script: bool,
            is_native: bool,
        }

        let mut file = File::open(filename).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let funs: Vec<FunctionEntry> =
            serde_json::from_str(&contents).expect("JSON was not well-formatted");

        let mut simple_env: SimpleEnv = Default::default();

        for entry in funs {
            let module_or_script = SimpleEnv::get_module_name_of(&entry.name);
            if module_or_script == "Script" {
                simple_env.scripts.insert(entry.name.clone());
            } else {
                simple_env.functions.insert(entry.name.clone());
                simple_env.modules.insert(module_or_script);
                if entry.is_public {
                    if entry.is_called_by_script {
                        simple_env.true_public_functions.insert(entry.name.clone());
                    } else {
                        simple_env.public_functions.insert(entry.name.clone());
                    }
                } else {
                    simple_env.private_functions.insert(entry.name.clone());
                }
                if entry.is_native {
                    simple_env.native_functions.insert(entry.name.clone());
                }
                simple_env
                    .callers
                    .insert(entry.name.clone(), HashSet::from_iter(entry.called_by));
            }
            simple_env
                .callees
                .insert(entry.name.clone(), HashSet::from_iter(entry.calls));
        }
        simple_env
    }

    pub fn get_module_name_of(func_name: &str) -> String {
        let idx = func_name.find("::").unwrap();
        func_name[..idx].to_string()
    }

    pub fn is_native(&self, func_name: &str) -> bool {
        self.native_functions.contains(func_name)
    }

    pub fn is_public(&self, func_name: &str) -> bool {
        self.public_functions.contains(func_name)
    }

    pub fn is_private(&self, func_name: &str) -> bool {
        !self.is_public(func_name)
    }

    pub fn is_script(&self, func_name: &str) -> bool {
        SimpleEnv::get_module_name_of(func_name) == "Script"
    }

    pub fn is_called_by_script(&self, func_name: &str) -> bool {
        self.true_public_functions.contains(func_name)
    }

    pub fn get_all_scripts(&self) -> HashSet<String> {
        self.scripts.clone()
    }

    pub fn get_all_modules(&self) -> HashSet<String> {
        self.modules.clone()
    }

    pub fn get_all_functions(&self) -> HashSet<String> {
        self.functions.clone()
    }

    pub fn get_all_internal_public_functions(&self) -> HashSet<String> {
        self.public_functions.clone()
    }

    pub fn get_all_public_functions(&self) -> HashSet<String> {
        self.true_public_functions.clone()
    }

    pub fn get_all_native_functions(&self) -> HashSet<String> {
        self.native_functions.clone()
    }

    pub fn get_all_private_functions(&self) -> HashSet<String> {
        self.private_functions.clone()
    }

    pub fn get_callers_of(&self, func_name: &str) -> HashSet<String> {
        self.callers.get(func_name).unwrap().clone()
    }

    pub fn get_callers_of_hashset(&self, func_name: &str) -> HashSet<String> {
        self.callers.get(func_name).unwrap().clone()
    }

    pub fn get_callees_of(&self, func_name: &str) -> HashSet<String> {
        self.callees.get(func_name).unwrap().clone()
    }

    pub fn get_functions_in(&self, module_name: &str) -> HashSet<String> {
        self.functions
            .iter()
            .filter(|func| SimpleEnv::get_module_name_of(func) == module_name)
            .cloned()
            .collect()
    }

    pub fn get_reachable_set_from(&self, func_name: &str) -> HashSet<String> {
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();

        visited.insert(func_name.to_string());
        queue.push_back(func_name.to_string());

        while let Some(func) = queue.pop_front() {
            for callee in self.get_callees_of(&func) {
                if !visited.contains(&callee) {
                    visited.insert(callee.clone());
                    queue.push_back(callee.clone());
                }
            }
        }
        visited
    }
}
