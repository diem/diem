mod reboot_random_validator;

pub use reboot_random_validator::RebootRandomValidator;

pub trait Experiment {
    fn run(&self) -> failure::Result<()>;
}
