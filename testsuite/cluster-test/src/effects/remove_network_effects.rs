/// RemoveNetworkEffect deletes all network effects introduced on an instance
use crate::{effects::Action, instance::Instance};
use failure;
use std::fmt;

pub struct RemoveNetworkEffects {
    instance: Instance,
}

impl RemoveNetworkEffects {
    pub fn new(instance: Instance) -> Self {
        Self { instance }
    }
}

impl Action for RemoveNetworkEffects {
    fn apply(&self) -> failure::Result<()> {
        println!("RemoveNetworkEffects for {}", self.instance);
        self.instance
            .run_cmd(vec!["sudo tc qdisc delete dev eth0 root".to_string()])
    }

    fn is_complete(&self) -> bool {
        true
    }
}

impl fmt::Display for RemoveNetworkEffects {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RemoveNetworkEffects for {}", self.instance)
    }
}
