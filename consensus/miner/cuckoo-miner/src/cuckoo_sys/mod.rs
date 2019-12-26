// Copyright 2017 The Grin Developers
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

#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![warn(missing_docs)]

//! Crate containing the low level calls to cuckoo-miner plugins, including
//! functions
//! for loading and unloading plugins, querying what plugins are installed on
//! the system,
//! as well as the actual mining calls to a plugin. This crate should be used
//! by other
//! cuckoo-miner crates, but should not be exposed to external consumers of the
//! crate.

pub mod ffi;
