// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use rand::{distributions::Alphanumeric, rngs::StdRng, Rng};

pub fn random_string(rng: &mut StdRng, len: usize) -> String {
    if len == 0 {
        "".to_string()
    } else {
        let mut string = "a".to_string();
        (1..len).for_each(|_| string.push(char::from(rng.sample(Alphanumeric))));
        string
    }
}
