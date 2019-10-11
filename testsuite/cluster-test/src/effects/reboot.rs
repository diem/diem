use crate::{effects::Action, instance::Instance};
use failure;
use std::fmt;

pub struct Reboot {
    instance: Instance,
}

impl Reboot {
    pub fn new(instance: Instance) -> Self {
        Self { instance }
    }
}

impl Action for Reboot {
    fn apply(&self) -> failure::Result<()> {
        println!("Rebooting {}", self.instance);
        self.instance.run_cmd(vec![
            "touch /dev/shm/cluster_test_reboot; nohup sudo /usr/sbin/reboot &",
        ])
    }

    fn is_complete(&self) -> bool {
        match self
            .instance
            .run_cmd(vec!["! cat /dev/shm/cluster_test_reboot"])
        {
            Ok(..) => {
                println!("Rebooting {} complete", self.instance);
                true
            }
            Err(..) => {
                println!(
                    "Rebooting {} in progress - did not reboot yet",
                    self.instance
                );
                false
            }
        }
    }
}

impl fmt::Display for Reboot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Reboot {}", self.instance)
    }
}
