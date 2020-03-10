// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::json_log::JsonLogEntry;
use std::time::Instant;

pub const TRACE_EVENT: &str = "trace_event";
pub const TRACE_EDGE: &str = "trace_edge";

#[macro_export]
macro_rules! trace_event {
    ($stage:expr, $node:tt) => {
        trace_event!($stage; {$crate::format_node!($node), module_path!(), Option::<u64>::None})
    };
    ($stage:expr; {$node:expr, $path:expr, $duration:expr}) => {
        $crate::json_log::send_json_log($crate::json_log::JsonLogEntry::new(
            $crate::libra_trace::TRACE_EVENT,
            serde_json::json!({
               "path": $path,
               "node": $node,
               "stage": $stage,
               "duration": $duration,
            }),
        ));
    }
}

#[macro_export]
macro_rules! trace_code_block {
    ($stage:expr, $node:tt) => {
        let trace_guard = $crate::libra_trace::TraceBlockGuard::new_entered(
            concat!($stage, "::done"),
            $crate::format_node!($node),
            module_path!(),
        );
        trace_event!($stage, $node);
    };
    ($stage:expr, $node:tt, $guard_vec:tt) => {
        let trace_guard = $crate::libra_trace::TraceBlockGuard::new_entered(
            concat!($stage, "::done"),
            $crate::format_node!($node),
            module_path!(),
        );
        trace_event!($stage, $node);
        $guard_vec.push(trace_guard);
    };
}

pub struct TraceBlockGuard {
    stage: &'static str,
    node: String,
    module_path: &'static str,
    started: Instant,
}

impl TraceBlockGuard {
    pub fn new_entered(
        stage: &'static str,
        node: String,
        module_path: &'static str,
    ) -> TraceBlockGuard {
        let started = Instant::now();
        TraceBlockGuard {
            stage,
            node,
            module_path,
            started,
        }
    }
}

impl Drop for TraceBlockGuard {
    fn drop(&mut self) {
        let duration = format!("{:.0?}", Instant::now().duration_since(self.started));
        trace_event!(self.stage; {self.node, self.module_path, duration});
    }
}

#[macro_export]
macro_rules! end_trace {
    ($stage:expr, $node:tt) => {
        $crate::json_log::send_json_log($crate::json_log::JsonLogEntry::new(
            $crate::libra_trace::TRACE_EVENT,
            serde_json::json!({
               "path": module_path!(),
               "node": $crate::format_node!($node),
               "stage": $stage,
               "end": true,
            }),
        ));
    };
}

#[macro_export]
macro_rules! trace_edge {
    ($stage:expr, $node_from:tt, $node_to:tt) => {
        $crate::json_log::send_json_log($crate::json_log::JsonLogEntry::new(
            $crate::libra_trace::TRACE_EDGE,
            serde_json::json!({
               "path": module_path!(),
               "node": $crate::format_node!($node_from),
               "node_to": $crate::format_node!($node_to),
               "stage": $stage,
            }),
        ));
    };
}

#[macro_export]
macro_rules! format_node {
    ({$($node_part:expr),+}) => {
        format!($crate::__trace_fmt_gen!($($node_part),+), $($node_part),+)
    }
}

// Internal helper macro
// Transforms (expr, expr, ...) into "{}::{}::..."
#[macro_export]
macro_rules! __trace_fmt_gen {
    ($p:expr) => {"{}"};
    ($p:expr, $($par:expr),+) => {concat!("{}::", $crate::__trace_fmt_gen!($($par),+))}
}

pub fn random_node(entries: &[JsonLogEntry], f_stage: &str, prefix: &str) -> Option<String> {
    for entry in entries {
        if entry.name != TRACE_EVENT {
            continue;
        }
        let node = entry
            .json
            .get("node")
            .expect("TRACE_EVENT::node not found")
            .as_str()
            .expect("TRACE_EVENT::node is not a string");
        let stage = entry
            .json
            .get("stage")
            .expect("TRACE_EVENT::stage not found")
            .as_str()
            .expect("TRACE_EVENT::stage is not a string");
        if node.starts_with(prefix) && stage == f_stage {
            return Some(node.to_string());
        }
    }
    None
}

pub fn trace_node(entries: &[JsonLogEntry], node_name: &str) {
    let mut nodes = vec![];
    nodes.push(node_name);
    for entry in entries {
        if entry.name != TRACE_EDGE {
            continue;
        }
        let node_from = entry
            .json
            .get("node")
            .expect("TRACE_EDGE::node not found")
            .as_str()
            .expect("TRACE_EDGE::node is not a string");
        if nodes.contains(&node_from) {
            let node_to = entry
                .json
                .get("node_to")
                .expect("TRACE_EDGE::node_to not found")
                .as_str()
                .expect("TRACE_EDGE::node_to is not a string");
            nodes.push(node_to);
        }
    }
    let mut start_time = None;
    for entry in entries {
        if !entry.name.starts_with("trace_") {
            continue;
        }
        let node = entry
            .json
            .get("node")
            .expect("TRACE_EVENT::node not found")
            .as_str()
            .expect("TRACE_EVENT::node is not a string");
        if !nodes.contains(&node) {
            continue;
        }

        let ts = entry.timestamp as u64;
        let peer = entry
            .json
            .get("peer")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if start_time.is_none() {
            start_time = Some(ts);
        }
        let trace_time = ts - start_time.unwrap();
        let stage = entry
            .json
            .get("stage")
            .expect("::stage not found")
            .as_str()
            .expect("::stage is not a string");
        let path = entry
            .json
            .get("path")
            .expect("::path not found")
            .as_str()
            .expect("::path is not a string");
        let duration = entry.json.get("duration").and_then(|v| v.as_str());
        let crate_name = crate_name(path);
        match entry.name {
            TRACE_EVENT => {
                let node = entry
                    .json
                    .get("node")
                    .expect("TRACE_EVENT::node not found")
                    .as_str()
                    .expect("TRACE_EVENT::node is not a string");
                if nodes.contains(&node) {
                    let end = entry.json.get("end").and_then(|m| m.as_bool());
                    let end_str = end.map_or("", |f| if f { " *end" } else { "" });
                    let duration_str = duration.map_or("".to_string(), |d| format!(" [{}]", d));

                    println!(
                        "{}[{:^11}] +{:05} {:*^10} {} {}{}{}{}",
                        crate_color(crate_name),
                        crate_name,
                        trace_time,
                        peer,
                        node,
                        stage,
                        duration_str,
                        end_str,
                        reset_color()
                    );
                    if end == Some(true) {
                        return;
                    }
                }
            }
            TRACE_EDGE => {
                let node_to = entry
                    .json
                    .get("node_to")
                    .expect("TRACE_EDGE::node_to not found")
                    .as_str()
                    .expect("TRACE_EDGE::node_to is not a string");
                println!(
                    "{}[{:^11}] +{:05} {:*^10} {}->{} {}{}",
                    crate_color(crate_name),
                    crate_name,
                    trace_time,
                    peer,
                    node,
                    node_to,
                    stage,
                    reset_color()
                );
            }
            _ => {}
        }
    }
}

fn reset_color() -> &'static str {
    "\x1B[K\x1B[49m"
}

fn crate_color(path: &str) -> &'static str {
    match path {
        "consensus" => "\x1B[43m",
        "mempool" => "\x1B[46m",
        "executor" => "\x1B[104m",
        "ac" => "\x1B[103m",
        "json_rpc" => "\x1B[103m",
        "vm" => "\x1B[45m",
        _ => "\x1B[49m",
    }
}

fn crate_name(path: &str) -> &str {
    let name = match path.find("::") {
        Some(pos) => &path[0..pos],
        None => path,
    };
    let name = if name.starts_with("libra_") {
        &name["libra_".len()..]
    } else {
        name
    };
    abbreviate_crate(name)
}

fn abbreviate_crate(name: &str) -> &str {
    match name {
        "admission_control_service" => "ac",
        _ => name,
    }
}
