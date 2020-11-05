mod simple_env;

use simple_env::SimpleEnv;
use std::collections::{hash_map::Entry, HashMap, HashSet};

fn main() {
    let env = simple_env::SimpleEnv::from_json("functions.json");
    summary(&env);
    scripts_to_callers(&env);
    modules_breakup(&env);
    //scripts_to_event_emitting_functions(&env);
}

fn summary(env: &SimpleEnv) {
    println!(">> SUMMARY");

    println!("Scripts: {}", env.get_all_scripts().len());
    println!("Modules: {}", env.get_all_modules().len());
    println!(
        "Functions: {} - Natives: {}",
        env.get_all_functions().len(),
        env.get_all_native_functions().len(),
    );
    println!(
        "Public Function: {}",
        env.get_all_public_functions().len()
    );
    println!(
        "Private Function: {}",
        env.get_all_private_functions().len()
    );
    println!(
        "Internal Public Function: {}",
        env.get_all_internal_public_functions().len()
    );
}

fn scripts_to_callers(env: &SimpleEnv) {
    println!("\n>> SCRIPTS TO CALLERS");

    let mut called_modules: HashMap<String, u64> = HashMap::new();
    let mut called_functions: HashMap<String, u64> = HashMap::new();
    for func in env.get_all_functions() {
        if env.is_called_by_script(&func) {
            let module_name = SimpleEnv::get_module_name_of(&func);
            match called_modules.entry(module_name) {
                Entry::Occupied(mut entry) => *entry.get_mut() += 1,
                Entry::Vacant(mut entry) => {
                    entry.insert(1);
                    ()
                }
            }
            match called_functions.entry(func.clone()) {
                Entry::Occupied(mut entry) => *entry.get_mut() += 1,
                Entry::Vacant(mut entry) => {
                    entry.insert(1);
                    ()
                }
            }
        }
    }
    println!("Modules called by Scripts: {}", called_modules.len());
    let mut sorted_modules = called_modules.iter().collect::<Vec<(&String, &u64)>>();
    sorted_modules.sort_by(|(_, count1), (_, count2)| count2.partial_cmp(count1).unwrap());
    for (module, count) in sorted_modules {
        println!("\t{} ({})", module, count);
    }
    println!("Functions called by Scripts: {}", called_functions.len());
    let mut sorted_functions = called_functions.iter().collect::<Vec<(&String, &u64)>>();
    sorted_functions.sort_by(|(name1, _), (name2, _)| name1.partial_cmp(name2).unwrap());
    for (func, count) in sorted_functions {
        println!("\t{} ({})", func, count);
    }
}

fn modules_breakup(env: &SimpleEnv) {
    println!("\n>> MODULES BREAKUP");

    let basic_modules = vec![
        "Vector",
        "Option",
        "Errors",
        "Signer",
        "Roles",
        "FixedPoint32",
        "Hash",
        "CoreAddresses",
        "Event",
        "LibraTimestamp",
    ];

    println!("* Standalone Modules: {}", basic_modules.len());
    for basic in &basic_modules {
        println!("\t{}", basic);
    }

    let mut module_coupling = HashMap::new();
    for module in env.get_all_modules() {
        if basic_modules.contains(&module.as_str()) {
            continue;
        }
        let mut called_modules= HashSet::new();
        for func in env.get_all_functions() {
            if SimpleEnv::get_module_name_of(&func) != module {
                continue;
            }
            for callee in env.get_callees_of(func.as_str()) {
                let owner = SimpleEnv::get_module_name_of(callee.as_str());
                if owner == module || owner == "Script" || basic_modules.contains(&owner.as_str()) {
                    continue;
                }
                called_modules.insert(owner);
            }
        }
        module_coupling.insert(module, called_modules);
    }

    println!("* Module Couplings - module (called modules count)");
    let mut sorted_coupling = module_coupling.iter().collect::<Vec<(&String, &HashSet<String>)>>();
    sorted_coupling.sort_by(|(_, modules1), (_, modules2)| modules2.len().partial_cmp(&modules1.len()).unwrap());
    for (module, called_modules) in sorted_coupling {
        println!("\t{} ({})", module, called_modules.len());
    }

    println!("* Callers of modules with no dependencies");
    let mut no_deps_callers = HashMap::new();
    for (no_deps_module, _) in module_coupling.iter().filter(|(_, called_modules)| called_modules.is_empty()) {
        let mut caller_modules= HashSet::new();
        for func in env.get_all_functions() {
            if &SimpleEnv::get_module_name_of(&func) != no_deps_module {
                continue;
            }
            for caller in env.get_callers_of(func.as_str()) {
                let owner = SimpleEnv::get_module_name_of(caller.as_str());
                if &owner == no_deps_module {
                    continue;
                }
                caller_modules.insert(owner);
            }
        }
        no_deps_callers.insert(no_deps_module.to_string(), caller_modules);
    }
    let mut sorted_no_deps_callers = no_deps_callers.iter().collect::<Vec<(&String, &HashSet<String>)>>();
    sorted_no_deps_callers.sort_by(|(_, modules1), (_, modules2)| modules2.len().partial_cmp(&modules1.len()).unwrap());
    for (module, caller_modules) in sorted_no_deps_callers {
        println!("\t{} ({})", module, caller_modules.len());
        for caller in caller_modules {
            println!("\t\t{}", caller);
        }
    }

}

fn scripts_to_event_emitting_functions(env: &SimpleEnv) {
    let callers = env.get_callers_of("Event::emit_event");
    for script in env.get_all_scripts() {
        let reachable_set = env.get_reachable_set_from(&script);
        println!("{}\n{:?}\n", script, reachable_set.intersection(&callers));
    }
}
