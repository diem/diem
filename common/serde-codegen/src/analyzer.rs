// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_reflection::{ContainerFormat, Format, FormatHolder, RegistryOwned, Result};
use std::collections::{BTreeMap, BTreeSet, HashSet};

fn get_dependencies(format: &ContainerFormat) -> Result<BTreeSet<&str>> {
    let mut result = BTreeSet::new();
    format.visit(&mut |format| {
        if let Format::TypeName(x) = format {
            result.insert(x.as_str());
        }
        Ok(())
    })?;
    Ok(result)
}

pub fn get_dependency_map(registry: &RegistryOwned) -> Result<BTreeMap<&str, BTreeSet<&str>>> {
    let mut children = BTreeMap::new();
    for (name, format) in registry {
        children.insert(name.as_str(), get_dependencies(format)?);
    }
    Ok(children)
}

pub fn best_effort_topological_sort<'a>(
    children: &BTreeMap<&'a str, BTreeSet<&'a str>>,
) -> Result<Vec<&'a str>> {
    let mut queue: Vec<_> = children.keys().cloned().collect();
    // Pre-sort nodes by increasing number of children.
    queue.sort_by(|node1, node2| children[node1].len().cmp(&children[node2].len()));

    let mut result = Vec::new();
    // Nodes already inserted in result.
    let mut sorted = HashSet::new();
    // Nodes for which children have been enqueued.
    let mut seen = HashSet::new();

    while let Some(node) = queue.pop() {
        if sorted.contains(&node) {
            continue;
        }
        if seen.contains(&node) {
            // Second time we see this node.
            // By now, children should have been sorted.
            sorted.insert(node);
            result.push(node);
            continue;
        }
        // First time we see this node:
        // 1. Mark it so that it is no longer enqueued.
        seen.insert(node);
        // 2. Schedule all the (yet unseen) children then this node for a second visit.
        queue.push(node);
        for child in &children[&node] {
            if !seen.contains(child) {
                queue.push(child);
            }
        }
    }
    Ok(result)
}
