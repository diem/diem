/// PacketLoss introduces a given percentage of PacketLoss for a given instance
use crate::{effects::Action, instance::Instance};
use failure;
use std::fmt;

pub struct PacketLoss {
    instance: Instance,
    percent: f32,
}

impl PacketLoss {
    pub fn new(instance: Instance, percent: f32) -> Self {
        Self { instance, percent }
    }
}

impl Action for PacketLoss {
    fn apply(&self) -> failure::Result<()> {
        println!("PacketLoss {:.*}% for {}", 2, self.percent, self.instance);
        self.instance.run_cmd(vec![format!(
            "sudo tc qdisc add dev eth0 root netem loss {:.*}%",
            2, self.percent
        )])
    }

    fn is_complete(&self) -> bool {
        true
    }
}

impl fmt::Display for PacketLoss {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PacketLoss {:.*}% for {}",
            2, self.percent, self.instance
        )
    }
}
