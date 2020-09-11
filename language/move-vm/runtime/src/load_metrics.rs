use libra_metrics::{register_histogram, Histogram};
use once_cell::sync::Lazy;

//*******************Histogram Timers*********************//
pub static MOVE_VM_LOAD_MODULE_EXPECT_NO_MISSING_DEPENDENCIES_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "MOVE_VM_LOAD_MODULE_EXPECT_NO_MISSING_DEPENDENCIES_SECONDS",
        // metric description
        "The time spent in loading a module by Libra VM"
    )
    .unwrap()
});

pub static MOVE_VM_MODULE_CACHE_RESOLVE_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "MOVE_VM_MODULE_CACHE_RESOLVE_SECONDS",
        // metric description
        "The time spent in resolving module cache by Libra VM"
    )
    .unwrap()
});

pub static MOVE_VM_MODULE_CACHE_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "MOVE_VM_MODULE_CACHE_SECONDS",
        // metric description
        "The time spent in loading module cache by Libra VM"
    )
    .unwrap()
});

pub static MOVE_VM_LOAD_TYPE_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "MOVE_VM_LOAD_TYPE_SECONDS",
        // metric description
        "The time spent in loading types"
    )
    .unwrap()
});


pub static MOVE_VM_VERIFY_TYPE_PARAMETERS_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "MOVE_VM_VERIFY_TYPE_PARAMETERS_SECONDS",
        // metric description
        "The time spent by Move VM to verify type arguments"
    )
    .unwrap()
});


pub static MOVE_VM_LOAD_MODULE_CACHE_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "MOVE_VM_LOAD_MODULE_CACHE_SECONDS",
        // metric description
        "The time spent to load module cache"
    )
    .unwrap()
});


pub static MOVE_VM_DATA_STORE_LOAD_MODULE_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "MOVE_VM_DATA_STORE_LOAD_MODULE_SECONDS",
        // metric description
        "The time spent to load the data store module"
    )
    .unwrap()
});

pub static MOVE_VM_DESERIALIZE_AND_VERIFY_MODULE_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "MOVE_VM_DESERIALIZE_AND_VERIFY_MODULE_SECONDS",
        // metric description
        "The time spent to deserialize and verify a module"
    )
    .unwrap()
});


pub static MOVE_VM_LOAD_FUNCTION_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "MOVE_VM_LOAD_FUNCTION_SECONDS",
        // metric description
        "The time spent to load a function"
    )
    .unwrap()
});
