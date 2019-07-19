use crate::instance::Instance;
use failure::{self, prelude::*};
use rand::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub struct Cluster {
    instances: Vec<Instance>, // guaranteed non-empty
}

impl Cluster {
    pub fn discover() -> failure::Result<Self> {
        let f = File::open("instances.txt")?;
        let f = BufReader::new(f);
        let mut instances = vec![];
        for line in f.lines() {
            instances.push(Instance::new(line?));
        }
        ensure!(!instances.is_empty(), "instances.txt is empty");
        Ok(Self { instances })
    }

    pub fn random_instance(&self) -> Instance {
        let mut rnd = rand::thread_rng();
        self.instances.choose(&mut rnd).unwrap().clone()
    }
}
