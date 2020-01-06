// Copyright 2017-2020 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Tests exercising the loading and unloading of plugins, as well as the
/// existence and correct functionality of each plugin function
mod common;

extern crate cuckoo_miner as cuckoo;
extern crate rand;

use common::{mine_async_for_duration, mining_plugin_dir_for_tests};
use cuckoo::PluginConfig;

// AT LEAN ///////////////
#[test]
fn mine_cuckatoo_lean_compat_cpu_19() {
    let mut config =
        PluginConfig::new(mining_plugin_dir_for_tests(), "cuckatoo_lean_cpu_compat_19").unwrap();
    config.params.nthreads = 4;
    mine_async_for_duration(&vec![config], 20);
}

#[ignore]
#[test]
fn mine_cuckatoo_lean_compat_cpu_31() {
    let mut config =
        PluginConfig::new(mining_plugin_dir_for_tests(), "cuckatoo_lean_cpu_compat_31").unwrap();
    config.params.nthreads = 4;
    mine_async_for_duration(&vec![config], 20);
}

#[ignore]
#[cfg(feature = "test-avx2")]
#[test]
fn mine_cuckatoo_lean_avx2_cpu_31() {
    let mut config =
        PluginConfig::new(mining_plugin_dir_for_tests(), "cuckatoo_lean_cpu_avx2_31").unwrap();
    config.params.nthreads = 4;
    mine_async_for_duration(&vec![config], 20);
}

// AT MEAN ///////////////

#[test]
fn mine_cuckatoo_mean_cpu_compat_19() {
    let mut config =
        PluginConfig::new(mining_plugin_dir_for_tests(), "cuckatoo_mean_cpu_compat_19").unwrap();
    config.params.nthreads = 4;
    mine_async_for_duration(&vec![config], 20);
}

#[cfg(feature = "test-avx2")]
fn mine_cuckatoo_mean_avx2_cpu_19() {
    let mut config =
        PluginConfig::new(mining_plugin_dir_for_tests(), "cuckatoo_mean_cpu_avx2_19").unwrap();
    config.params.nthreads = 4;
    mine_async_for_duration(&vec![config], 20);
}

#[ignore]
#[test]
fn mine_cuckatoo_mean_compat_cpu_31() {
    let mut config =
        PluginConfig::new(mining_plugin_dir_for_tests(), "cuckatoo_mean_cpu_compat_31").unwrap();
    config.params.nthreads = 4;
    mine_async_for_duration(&vec![config], 20);
}

#[ignore]
#[cfg(feature = "test-avx2")]
#[test]
fn mine_cuckatoo_mean_avx2_cpu_31() {
    let mut config =
        PluginConfig::new(mining_plugin_dir_for_tests(), "cuckatoo_mean_cpu_avx2_31").unwrap();
    config.params.nthreads = 4;
    mine_async_for_duration(&vec![config], 20);
}

// AT MEAN CUDA ///////////////
#[cfg(feature = "build-cuda-plugins")]
#[test]
fn mine_cuckatoo_mean_cuda_19() {
    let config = PluginConfig::new(mining_plugin_dir_for_tests(), "cuckatoo_mean_cuda_19").unwrap();
    mine_async_for_duration(&vec![config], 20);
}

#[ignore]
#[cfg(feature = "build-cuda-plugins")]
#[test]
fn mine_cuckatoo_mean_cuda_31() {
    let config = PluginConfig::new(mining_plugin_dir_for_tests(), "cuckatoo_mean_cuda_31").unwrap();
    mine_async_for_duration(&vec![config], 20);
}

// AT LEAN CUDA ///////////////
#[cfg(feature = "build-cuda-plugins")]
#[test]
fn mine_cuckatoo_lean_cuda_19() {
    let config = PluginConfig::new(mining_plugin_dir_for_tests(), "cuckatoo_lean_cuda_19").unwrap();
    mine_async_for_duration(&vec![config], 20);
}

#[ignore]
#[cfg(feature = "build-cuda-plugins")]
#[test]
fn mine_cuckatoo_lean_cuda_31() {
    let config = PluginConfig::new(mining_plugin_dir_for_tests(), "cuckatoo_lean_cuda_31").unwrap();
    mine_async_for_duration(&vec![config], 20);
}

// AR CPU //////////////////////////
#[ignore]
#[test]
fn mine_cuckaroo_mean_cpu_compat_19() {
    let mut config =
        PluginConfig::new(mining_plugin_dir_for_tests(), "cuckaroo_cpu_compat_19").unwrap();
    config.params.nthreads = 4;
    mine_async_for_duration(&vec![config], 20);
}

#[ignore]
#[cfg(feature = "test-avx2")]
#[test]
fn mine_cuckaroo_mean_cpu_avx2_29() {
    let mut config =
        PluginConfig::new(mining_plugin_dir_for_tests(), "cuckaroo_cpu_avx_19").unwrap();
    config.params.nthreads = 4;
    mine_async_for_duration(&vec![config], 20);
}

#[test]
fn mine_cuckaroo_mean_cpu_compat_29() {
    let mut config =
        PluginConfig::new(mining_plugin_dir_for_tests(), "cuckaroo_cpu_compat_29").unwrap();
    config.params.nthreads = 4;
    mine_async_for_duration(&vec![config], 20);
}

#[test]
fn mine_cuckarood_mean_cpu_compat_29() {
    let mut config =
        PluginConfig::new(mining_plugin_dir_for_tests(), "cuckarood_cpu_compat_29").unwrap();
    config.params.nthreads = 4;
    mine_async_for_duration(&vec![config], 20);
}

#[cfg(feature = "test-avx2")]
#[test]
fn mine_cuckaroo_mean_cpu_avx2_29() {
    let mut config =
        PluginConfig::new(mining_plugin_dir_for_tests(), "cuckaroo_cpu_avx_29").unwrap();
    config.params.nthreads = 4;
    mine_async_for_duration(&vec![config], 20);
}

#[cfg(feature = "test-avx2")]
#[test]
fn mine_cuckarood_mean_cpu_avx2_29() {
    let mut config =
        PluginConfig::new(mining_plugin_dir_for_tests(), "cuckarood_cpu_avx_29").unwrap();
    config.params.nthreads = 4;
    mine_async_for_duration(&vec![config], 20);
}

#[cfg(feature = "build-cuda-plugins")]
#[test]
fn mine_cuckaroo_cuda_29() {
    let config = PluginConfig::new(mining_plugin_dir_for_tests(), "cuckaroo_cuda_29").unwrap();
    mine_async_for_duration(&vec![config], 20);
}

#[cfg(feature = "build-cuda-plugins")]
#[test]
fn mine_cuckarood_cuda_29() {
    let config = PluginConfig::new(mining_plugin_dir_for_tests(), "cuckarood_cuda_29").unwrap();
    mine_async_for_duration(&vec![config], 20);
}

#[cfg(feature = "build-cuda-plugins")]
#[test]
fn mine_cuckaroom_cuda_29() {
    let config = PluginConfig::new(mining_plugin_dir_for_tests(), "cuckaroom_cuda_29").unwrap();
    mine_async_for_duration(&vec![config], 20);
}
