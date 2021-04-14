// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Functions for running benchmarks and storing the results as files, and for reading
// those results and plotting them.

use anyhow::anyhow;
use bytecode::options::ProverOptions;
use clap::{App, Arg};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use itertools::Itertools;
use log::LevelFilter;
use move_model::{
    model::{FunctionEnv, GlobalEnv, ModuleEnv, VerificationScope},
    run_model_builder,
};
use move_prover::{
    check_errors, cli::Options, create_and_process_bytecode, generate_boogie, verify_boogie,
};
use plotters::{
    coord::types::RangedCoordu32,
    evcxr::{evcxr_figure, SVGWrapper},
    prelude::{
        Cartesian2d, IntoFont, RGBColor, Rectangle, ShapeStyle, Text, BLACK, BLUE, CYAN, GREEN,
        MAGENTA, RED, WHITE, YELLOW,
    },
};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{LineWriter, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

// ============================================================================================
// Running benchmarks

struct Runner {
    options: Options,
    out: LineWriter<File>,
    error_writer: StandardStream,
    per_function: bool,
}

pub fn benchmark(args: &[String]) {
    let cmd_line_parser = App::new("benchmark")
        .version("0.1.0")
        .about("Benchmark program for the Move Prover")
        .author("The Diem Core Contributors")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1)
                .value_name("CONFIG_PATH")
                .help(
                    "path to a prover toml configuration file. The benchmark output will be \
                          stored at `CONFIG_PATH.data. This can be repeated so different \
                          configurations are benchmarked against the same set of input modules.",
                ),
        )
        .arg(
            Arg::with_name("function")
                .short("f")
                .long("func")
                .help("whether benchmarking should happen per function; default is per module"),
        )
        .arg(
            Arg::with_name("dependencies")
                .long("dependency")
                .short("d")
                .multiple(true)
                .number_of_values(1)
                .takes_value(true)
                .value_name("PATH_TO_DEPENDENCY")
                .help(
                    "path to a Move file, or a directory which will be searched for \
                    Move files, containing dependencies which will not be verified",
                ),
        )
        .arg(
            Arg::with_name("sources")
                .multiple(true)
                .value_name("PATH_TO_SOURCE_FILE")
                .min_values(1)
                .help("the source files to verify"),
        );
    let matches = cmd_line_parser.get_matches_from(args);
    let get_vec = |s: &str| -> Vec<String> {
        match matches.values_of(s) {
            Some(vs) => vs.map(|v| v.to_string()).collect(),
            _ => vec![],
        }
    };
    let sources = get_vec("sources");
    let deps = get_vec("dependencies");
    let configs: Vec<Option<String>> = if matches.is_present("config") {
        get_vec("config").into_iter().map(Some).collect_vec()
    } else {
        vec![None]
    };
    let per_function = matches.is_present("function");

    for config_spec in configs {
        let (config, out) = if let Some(config_file) = &config_spec {
            let extension = if per_function { "fun_data" } else { "mod_data" };
            let out = PathBuf::from(config_file)
                .with_extension(extension)
                .to_string_lossy()
                .to_string();
            (config_spec, out)
        } else {
            (None, "benchmark.data".to_string())
        };
        if let Err(s) = run_benchmark(&out, config.as_ref(), &sources, &deps, per_function) {
            println!("ERROR: execution failed: {}", s);
        } else {
            println!("results stored at `{}`", out);
        }
    }
}

fn run_benchmark(
    out: &str,
    config_file_opt: Option<&String>,
    modules: &[String],
    dep_dirs: &[String],
    per_function: bool,
) -> anyhow::Result<()> {
    println!("building model");
    let env = run_model_builder(modules, dep_dirs)?;
    let mut error_writer = StandardStream::stderr(ColorChoice::Auto);
    let mut options = if let Some(config_file) = config_file_opt {
        Options::create_from_toml_file(config_file)?
    } else {
        Options::default()
    };
    options.verbosity_level = LevelFilter::Warn;
    options.backend.proc_cores = 1;
    options.backend.derive_options();
    options.setup_logging();
    check_errors(&env, &options, &mut error_writer, "unexpected build errors")?;

    let config_descr = if let Some(config) = config_file_opt {
        config.clone()
    } else {
        "default".to_string()
    };

    let mut out = LineWriter::new(File::create(out)?);

    writeln!(out, "# config: {}", config_descr)?;
    writeln!(out, "# time  : {}", chrono::Utc::now())?;

    let mut runner = Runner {
        out,
        options,
        per_function,
        error_writer,
    };
    println!(
        "Starting benchmarking with config `{}`.\n\
        Notice that execution is slow because we enforce single core execution.",
        config_descr
    );
    runner.bench(&env)
}

impl Runner {
    fn bench(&mut self, env: &GlobalEnv) -> anyhow::Result<()> {
        for module in env.get_modules() {
            if module.is_target() {
                if self.per_function {
                    for fun in module.get_functions() {
                        self.bench_function(fun)?;
                    }
                } else {
                    self.bench_module(module)?;
                }
            }
        }
        Ok(())
    }

    fn bench_function(&mut self, fun: FunctionEnv<'_>) -> anyhow::Result<()> {
        print!("benchmarking function {} ..", fun.get_full_name_str());
        std::io::stdout().flush()?;

        // Scope verification to the given function
        let env = fun.module_env.env;
        self.options.prover.verify_scope = VerificationScope::Only(fun.get_full_name_str());
        ProverOptions::set(env, self.options.prover.clone());
        // Run benchmark
        let (duration, status) = self.bench_function_or_module(fun.module_env.env)?;

        // Write data record of benchmark result
        writeln!(
            self.out,
            "{:<40} {:>12} {:>12}",
            fun.get_full_name_str(),
            duration.as_millis(),
            status
        )?;

        println!("\x08\x08{:.3}s {}.", duration.as_secs_f64(), status);
        Ok(())
    }

    fn bench_module(&mut self, module: ModuleEnv<'_>) -> anyhow::Result<()> {
        print!("benchmarking module {} ..", module.get_full_name_str());
        std::io::stdout().flush()?;

        // Scope verification to the given module
        self.options.prover.verify_scope =
            VerificationScope::OnlyModule(module.get_full_name_str());
        ProverOptions::set(module.env, self.options.prover.clone());

        // Run benchmark
        let (duration, status) = self.bench_function_or_module(module.env)?;

        // Write data record of benchmark result
        writeln!(
            self.out,
            "{:<40} {:>12} {:>12}",
            module.get_full_name_str(),
            duration.as_millis(),
            status
        )?;

        println!("\x08\x08{:.3}s {}.", duration.as_secs_f64(), status);
        Ok(())
    }

    fn bench_function_or_module(&mut self, env: &GlobalEnv) -> anyhow::Result<(Duration, String)> {
        // Create and process bytecode.
        let targets = create_and_process_bytecode(&self.options, env);
        check_errors(
            env,
            &self.options,
            &mut self.error_writer,
            "unexpected transformation errors",
        )?;

        // Generate boogie code.
        let code_writer = generate_boogie(&env, &self.options, &targets)?;
        check_errors(
            env,
            &self.options,
            &mut self.error_writer,
            "unexpected boogie generation errors",
        )?;

        // Verify boogie, measuring duration.
        let now = Instant::now();
        verify_boogie(&env, &self.options, &targets, code_writer)?;

        // Determine result status.
        let status = if env.error_count() > 0 {
            if env.has_diag("timeout") {
                "timeout"
            } else {
                "errors"
            }
        } else {
            "ok"
        };
        env.clear_diag();
        Ok((now.elapsed(), status.to_string()))
    }
}

// ============================================================================================
// Analyzing and plotting benchmarks

pub const LIGHT_GRAY: RGBColor = RGBColor(0xb4, 0xb4, 0xb4);
pub const MEDIUM_GRAY: RGBColor = RGBColor(0x90, 0x90, 0x90);
pub const GRAY: RGBColor = RGBColor(0x63, 0x63, 0x63);
pub const DARK_GRAY: RGBColor = RGBColor(0x49, 0x48, 0x48);

pub const GRAY_PALETTE: &[&RGBColor] = &[&LIGHT_GRAY, &MEDIUM_GRAY, &GRAY, &DARK_GRAY, &BLACK];
pub const COLOR_PALETTE: &[&RGBColor] = &[&GREEN, &BLUE, &RED, &CYAN, &YELLOW, &MAGENTA];

/// Represents a benchmark.
#[derive(Clone, Debug)]
pub struct Benchmark {
    /// The simple name of the configuration.
    pub config: String,
    /// The associated data.
    pub data: Vec<BenchmarkData>,
}

/// A data entry of a benchmark.
#[derive(Clone, Debug)]
pub struct BenchmarkData {
    pub name: String,
    pub duration: usize,
    pub status: String,
}

/// Read benchmark from data file.
pub fn read_benchmark(data_file: &str) -> anyhow::Result<Benchmark> {
    let config = PathBuf::from(data_file)
        .with_extension("") // remove extension
        .file_name() // use simple filename
        .ok_or_else(|| anyhow!("invalid data file name"))?
        .to_string_lossy()
        .to_string();
    let content = std::fs::read_to_string(data_file)?;
    let mut data = vec![];
    for line in content.lines() {
        if line.starts_with('#') {
            continue;
        }
        let parts = line.split_whitespace().collect_vec();
        if parts.len() != 3 {
            return Err(anyhow!("bad data entry"));
        }
        let name = parts[0].to_string();
        let duration = parts[1].parse::<usize>()?;
        let status = parts[2].to_string();
        data.push(BenchmarkData {
            name,
            duration,
            status,
        });
    }
    Ok(Benchmark { config, data })
}

impl Benchmark {
    /// Sort the benchmark data by longest duration.
    pub fn sort(&mut self) {
        self.data
            .sort_by(|d1, d2| d1.duration.cmp(&d2.duration).reverse());
    }

    /// Shrink benchmark to the first count samples.
    pub fn take(&mut self, count: usize) {
        self.data.truncate(count)
    }
}

pub fn stats_benchmarks(benchmarks: &[&Benchmark]) -> String {
    fn sum(benchmark: &Benchmark) -> f64 {
        benchmark
            .data
            .iter()
            .filter_map(|d| {
                if d.status == "ok" {
                    Some(d.duration as f64)
                } else {
                    None
                }
            })
            .sum()
    }
    let baseline = sum(&benchmarks[0]);
    let mut res = String::new();
    for benchmark in benchmarks {
        let factor = sum(benchmark) / baseline;
        res = format!("{} {}={:.3}", res, benchmark.config, factor);
    }
    res
}

/// Plot a set of benchmarks in JupyterLab.
/// TODO: we should pull this apart into multiple functions and also let the plotting work
///   against different backends. This is just a first experiment to see what is possible.
pub fn plot_benchmarks(benchmarks: &[&Benchmark]) -> SVGWrapper {
    #[derive(Clone, Copy)]
    enum Result {
        Duration(usize),
        Error(usize),
        Timeout,
    }
    // Join matching samples over all benchmarks. This maps from sample name
    // to a pair of configuration and duration, or whether its a timeout or an error.
    let mut joined: BTreeMap<&str, Vec<(&str, Result)>> = BTreeMap::new();
    for Benchmark { config, data } in benchmarks {
        for BenchmarkData {
            name,
            duration,
            status,
        } in data
        {
            match status.as_str() {
                "ok" => joined
                    .entry(name)
                    .or_default()
                    .push((config, Result::Duration(*duration))),
                "timeout" => joined
                    .entry(name)
                    .or_default()
                    .push((config, Result::Timeout)),
                _ => joined
                    .entry(name)
                    .or_default()
                    .push((config, Result::Error(*duration))),
            }
        }
    }

    // Rearrange samples in order of first benchmark.
    let joined = benchmarks[0]
        .data
        .iter()
        .map(|d| (d.name.as_str(), joined.get(d.name.as_str()).unwrap()))
        .collect_vec();

    // Compute maximal duration and data points, for correct scaling.
    let data_points = joined.len() as u32;
    let max_duration = joined
        .iter()
        .map(|(_, e)| e.iter().map(|(_, d)| *d))
        .flatten()
        .filter_map(|r| {
            if let Result::Duration(d) | Result::Error(d) = r {
                Some(d)
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0) as u32;

    // We are drawing data points as horizontal bars, therefore x-axis is max_duration
    // and y-axis datapoints.
    let real_x = 1000u32;
    let real_y = data_points * 60u32;
    evcxr_figure((real_x, real_y), |root| {
        let duration_percent = |p: usize| ((max_duration as f64) * (p as f64) / 100f64) as u32;
        let bar = |y: u32, w: u32, style| Rectangle::new([(0, y + 1), (w, y + 8)], style);
        let label = |s: &str, x: u32, y: u32, h| {
            Text::new(s.to_string(), (x, y), ("sans-serif", h).into_font())
        };
        let filled_shape = |i: usize| ShapeStyle::from(GRAY_PALETTE[i]).filled();
        let stroke_shape =
            |i: usize| ShapeStyle::from(GRAY_PALETTE[i]).stroke_width(duration_percent(1));

        let root = root.apply_coord_spec(Cartesian2d::<RangedCoordu32, RangedCoordu32>::new(
            0..max_duration + duration_percent(10), // + 10% for label
            0..(data_points + 1) * ((1 + benchmarks.len() as u32) * 10),
            (0..real_x as i32, 0..real_y as i32),
        ));
        root.fill(&WHITE)?;
        let mut ycoord = 0;

        // Draw legend
        for (i, benchmark) in benchmarks.iter().enumerate() {
            root.draw(&bar(ycoord, duration_percent(5), filled_shape(i)))?;
            root.draw(&label(
                &format!("= {}", benchmark.config),
                duration_percent(6),
                ycoord + 2,
                15.0,
            ))?;
            ycoord += 10;
        }
        ycoord += 10;
        // Draw samples.
        for (sample, variants) in joined {
            root.draw(&label(sample, 0, ycoord, 15.0))?;
            ycoord += 7;
            for (i, (_, result)) in variants.iter().enumerate() {
                let (weight, note, style) = match result {
                    Result::Duration(d) => (
                        *d as u32,
                        format!("{:.3}", (*d as f64) / 1000f64),
                        filled_shape(i),
                    ),
                    Result::Timeout => (max_duration, "timeout".to_string(), stroke_shape(i)),
                    Result::Error(d) => (*d as u32, "error".to_string(), filled_shape(i)),
                };
                root.draw(&bar(ycoord, weight, style))?;
                root.draw(&label(
                    &note,
                    weight + duration_percent(1),
                    ycoord + 2,
                    13.0,
                ))?;
                ycoord += 10;
            }
            ycoord += 3;
        }
        Ok(())
    })
}
