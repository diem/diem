use crate::instance::Instance;
use failure::{self, prelude::*};
use rand::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Clone)]
pub struct Cluster {
    instances: Vec<Instance>, // guaranteed non-empty
}

impl Cluster {
    pub fn discover() -> failure::Result<Self> {
        let f = File::open("instances.txt")?;
        let f = BufReader::new(f);
        let mut instances = vec![];
        for line in f.lines() {
            let line = line?;
            let split: Vec<&str> = line.split(':').collect();
            ensure!(
                split.len() == 2,
                "instances.txt has incorrect line: {}",
                line
            );
            instances.push(Instance::new(split[0].into(), split[1].into()));
        }
        ensure!(!instances.is_empty(), "instances.txt is empty");
        Ok(Self { instances })
    }

    pub fn random_instance(&self) -> Instance {
        let mut rnd = rand::thread_rng();
        self.instances.choose(&mut rnd).unwrap().clone()
    }

    pub fn instances(&self) -> &Vec<Instance> {
        &self.instances
    }
}
