// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use regex::Regex;
use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet, VecDeque},
    env, fs, io,
    path::{Path, PathBuf},
};

fn generate_diagram_per_module(
    graph: &BTreeMap<String, Vec<String>>,
    out_dir: &Path,
    is_forward: bool,
) -> anyhow::Result<()> {
    let empty_list = Vec::new();

    for (module, _) in graph.iter() {
        // (breadth-first) search and gather the modules in `graph` which are reachable from `module`.
        let mut dot_src: String = String::new();
        let mut visited: BTreeSet<String> = BTreeSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();

        visited.insert(module.clone());
        queue.push_back(module.clone());

        while let Some(d) = queue.pop_front() {
            let dep_list = graph.get(&d).unwrap_or(&empty_list);
            dot_src.push_str(&format!("    {}\n", d));
            for dep in dep_list.iter() {
                if is_forward {
                    dot_src.push_str(&format!("    {} -> {}\n", d, dep));
                } else {
                    dot_src.push_str(&format!("    {} -> {}\n", dep, d));
                }
                if !visited.contains(dep) {
                    visited.insert(dep.clone());
                    queue.push_back(dep.clone());
                }
            }
        }
        let out_file = out_dir.join(format!(
            "{}.{}.dot",
            module,
            (if is_forward { "forward" } else { "backward" }).to_string()
        ));

        fs::write(
            &out_file,
            format!("digraph G {{\n    rankdir=BT\n{}}}\n", dot_src),
        )?;
        println!("{:?}", out_file);
    }
    Ok(())
}

fn generate(inp_dir: &Path, out_dir: &Path) -> anyhow::Result<()> {
    let mut dep_graph: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut dep_graph_inverse: BTreeMap<String, Vec<String>> = BTreeMap::new();

    println!("The stdlib files detected:");
    for entry in fs::read_dir(inp_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() || path.extension().is_none() || !path.extension().unwrap().eq("move") {
            // skip anything other than .move file.
            continue;
        }
        println!("{:?}", path);
        let content = fs::read_to_string(path)?;

        let rex1 = Regex::new(r"(?m)^module\s+(\w+)\s*\{").unwrap();
        let caps = rex1.captures(&content);
        if caps.is_none() {
            // skip due to no module declaration found.
            continue;
        }
        let module_name = caps.unwrap()[1].to_string();
        if let Entry::Vacant(e) = dep_graph_inverse.entry(module_name.clone()) {
            e.insert(vec![]);
        }

        let mut dep_list: Vec<String> = Vec::new();
        // TODO: This is not 100% correct because modules can be used with full qualification and without `use`.
        // Refer to `move-prover/src/lib.rs` which correctly addresses this issue.
        let rex2 = Regex::new(r"(?m)use 0x1::(\w+)\s*(;|:)").unwrap();
        for cap in rex2.captures_iter(&content) {
            let dep = cap.get(1).unwrap().as_str().to_string();
            dep_list.push(dep.clone());

            match dep_graph_inverse.entry(dep) {
                Entry::Vacant(e) => {
                    e.insert(vec![module_name.clone()]);
                }
                Entry::Occupied(mut e) => {
                    e.get_mut().push(module_name.clone());
                }
            }
        }
        dep_graph.insert(module_name, dep_list);
    }
    fs::create_dir_all(out_dir)?;

    // Generate a .dot file for the entire dependency graph.
    let mut dot_src: String = String::new();

    for (module, dep_list) in dep_graph.iter() {
        dot_src.push_str(&format!("    {}\n", module));
        for dep in dep_list.iter() {
            dot_src.push_str(&format!("    {} -> {}\n", module, dep));
        }
    }

    fs::write(
        out_dir.join("(EntireGraph).dot"),
        format!("digraph G {{\n    rankdir=BT\n{}}}\n", dot_src),
    )?;

    println!(
        "\nThe diagram files (.dot) generated:\n{:?}",
        out_dir.join("(EntireGraph).dot")
    );

    // Generate a .forward.dot file for the forward dependency graph per module.
    generate_diagram_per_module(&dep_graph, out_dir, true)?;

    // Generate a .backward.dot file for the backward dependency graph per module.
    generate_diagram_per_module(&dep_graph_inverse, out_dir, false)?;

    println!(
        "\nTo convert these .dot files into .pdf files, run {:?}.",
        out_dir.join("convert_all_dot_to_pdf.sh")
    );

    Ok(())
}

fn locate_inp_out_dir() -> io::Result<(PathBuf, PathBuf)> {
    // locate the language directory
    let lang_dir = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("language");

    println!("The `language` directory is located as:\n{:?}\n", lang_dir);

    // construct the input and the output paths
    let inp_dir = lang_dir.join("stdlib/modules");
    let out_dir = lang_dir.join("move-prover/diagen/diagrams");

    Ok((inp_dir, out_dir))
}

fn main() {
    if let Ok((inp_dir, out_dir)) = locate_inp_out_dir() {
        let err = generate(inp_dir.as_path(), out_dir.as_path());
        if let Err(err) = err {
            println!("Error: {:?}", err);
        }
    } else {
        println!("Error: Cannot locate the language directory.");
    }
}
