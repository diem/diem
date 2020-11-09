// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_logger::prelude::*;
use once_cell::sync::Lazy;
use std::{sync::Mutex, time::Instant};

// A `Tracer` global to track time in execution of blocks.
// A block is a chunk of code "protected" by a `TraceBlockGen`. When a `TraceBlockGen`
// goes out of scope the time is computed from creation of the `TraceBlockGen`.
// The `Tracer` is cleaned up when either `print_tracer` (usually for tests) or
// `log_tracer` is invoked.
static TRACER: Lazy<Mutex<Tracer>> = Lazy::new(|| Mutex::new(Tracer::new()));

// Return a `TraceBlockGen` that is used for computing time. Time is computed
// when that instance goes out of scope. And the info is saved in the `Tracer`.
pub fn get_trace_block_gen(name: &'static str) -> TraceBlockGen {
    let id = { TRACER.lock().unwrap().lock_block_id() };
    let timer = Instant::now();
    TraceBlockGen { id, name, timer }
}

// Print the result of a tracing sequence. See `Tracer::coalesce`.
// If you call this API while TraceBlockGen are still on the stack things go bad and
// results may not be as expected.
pub fn print_tracer() {
    let tracer_clone = {
        let mut tracer = TRACER.lock().unwrap();
        let tracer_clone = tracer.clone();
        tracer.clear();
        tracer_clone
    };
    println!("{:#?}", tracer_clone.coalesce());
}

// Return a string with the result of a tracing sequence. See `Tracer::coalesce`.
// Typically used by a log macro. Helpful to relate taht to a given log in a given
// component context.
// If you call this API while TraceBlockGen are still on the stack things go bad and
// results may not be as expected.
pub fn log_tracer() -> String {
    let tracer_clone = {
        let mut tracer = TRACER.lock().unwrap();
        let tracer_clone = tracer.clone();
        tracer.clear();
        tracer_clone
    };
    format!("{:?}", tracer_clone.coalesce())
}

// A Tracer is a collection of `TraceBlock`.
// A Tracer should be more of a map from block_id to a collection of TraceBlock.
// TraceBlocks could even be arranged in a tree module to better represent the call
// graph. But that is all for another time and it has pretty significant implications
// and complications
#[derive(Debug, Clone)]
struct Tracer {
    blocks: Vec<Option<TraceBlock>>,
}

// A raceBlock is a name and a time spent to perform the operation represented by
// the given name
#[derive(Debug, Clone)]
struct TraceBlock {
    name: &'static str,
    time: u128,
}

// A TraceBlockGen carries an index in the Tracer where the result will be inserted.
// It has a name and an Instant used to compute final time.
#[derive(Debug)]
pub struct TraceBlockGen {
    id: usize,
    name: &'static str,
    timer: Instant,
}

// The result of a summary for a Tracer. It carries the name of the operation as defined in the
// TraceBlock, the count of how many operations were performed, the total time, the min
// and max of the time spent for the operation.
#[derive(Debug)]
struct TraceBlockInfo {
    name: &'static str,
    count: usize,
    total: u128,
    min: u128,
    max: u128,
    avg: u128,
    values: Vec<u128>,
    p5: u128,
    p25: u128,
    p50: u128,
    p75: u128,
    p95: u128,
}

impl Tracer {
    fn new() -> Self {
        Tracer { blocks: vec![] }
    }

    fn lock_block_id(&mut self) -> usize {
        let id = self.blocks.len();
        self.blocks.push(None);
        id
    }

    fn clear(&mut self) {
        self.blocks.clear();
    }

    // Iterate over the collection of TraceBlock and creates a single TraceBlockInfo per
    // operation (name of the TraceBlock). Computes, total, count, min and max.
    fn coalesce(&self) -> Vec<TraceBlockInfo> {
        let mut block_infos: Vec<TraceBlockInfo> = vec![];
        for block in &self.blocks {
            match block {
                None => error!("found None in TraceBlockInfo"),
                Some(block) => {
                    match block_infos
                        .iter_mut()
                        .find(|block_info| block_info.name == block.name)
                    {
                        None => block_infos.push(TraceBlockInfo::new(block.name, block.time)),
                        Some(block_info) => block_info.add(block),
                    }
                }
            }
        }
        for block_info in &mut block_infos {
            block_info.percentiles();
        }
        block_infos
    }
}

// Drop implementation for a TraceBlockGen, it computes a TraceBlock and it inserts
// it in the collection of TraceBlocks in the Tracer.
impl Drop for TraceBlockGen {
    fn drop(&mut self) {
        let time = self.timer.elapsed().as_micros();
        let name = self.name;
        let trace_block = TraceBlock { name, time };
        if let Some(val) = TRACER.lock().unwrap().blocks.get_mut(self.id) {
            *val = Some(trace_block);
        } else {
            error!("no value at index {}", self.id);
        }
    }
}

impl TraceBlockInfo {
    fn new(name: &'static str, value: u128) -> Self {
        TraceBlockInfo {
            name,
            count: 1,
            total: value,
            min: value,
            max: value,
            avg: 0,
            values: vec![value],
            p5: 0,
            p25: 0,
            p50: 0,
            p75: 0,
            p95: 0,
        }
    }

    #[allow(clippy::integer_arithmetic)]
    fn add(&mut self, block: &TraceBlock) {
        self.count += 1;
        self.total += block.time;
        self.min = std::cmp::min(self.min, block.time);
        self.max = std::cmp::max(self.max, block.time);
        self.values.push(block.time);
    }

    #[allow(clippy::integer_arithmetic)]
    fn percentiles(&mut self) {
        let len = self.values.len();
        if len == 0 {
            return;
        }
        for value in &self.values {
            self.avg += value;
        }
        self.avg /= len as u128;
        self.values.sort_unstable();
        let idx5 = len * 5 / 100;
        self.p5 = self.values[idx5];
        let idx25 = len * 25 / 100;
        self.p25 = self.values[idx25];
        let idx50 = len * 50 / 100;
        self.p50 = self.values[idx50];
        let idx75 = len * 75 / 100;
        self.p75 = self.values[idx75];
        let idx95 = len * 95 / 100;
        self.p95 = self.values[idx95];
        self.values = vec![];
    }
}
