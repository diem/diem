use crate::{effects::Effect, instance::Instance};
use failure;

pub struct Reboot {
    instance: Instance,
}

impl Reboot {
    pub fn new(instance: Instance) -> Self {
        Self { instance }
    }
}

impl Effect for Reboot {
    fn apply(&self) -> failure::Result<()> {
        println!("Rebooting {}", self.instance);
        self.instance.run_cmd(vec![
            "touch /dev/shm/cluster_test_reboot; nohup sudo /usr/sbin/reboot &",
        ])
    }

    fn is_complete(&self) -> bool {
        if self.instance.check_ac_port() {
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
        } else {
            println!(
                "Rebooting {} in progress - waiting for connection",
                self.instance
            );
            false
        }
    }
}
