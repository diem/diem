use libra_metrics::{register_histogram, Histogram};
use once_cell::sync::Lazy;

//*******************Histogram Timers*********************//
pub static LIBRA_MOVEVM_LOAD_MODULE_EXPECT_NO_MISSING_DEPENDENCIES_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_movevm_module_expect_no_missing_dependencies_seconds",
        // metric description
        "The time spent in loading a module by Libra VM"
    )
    .unwrap()
});

pub static LIBRA_MOVEVM_MODULE_CACHE_RESOLVE_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_movevm_module_cache_reslve_seconds",
        // metric description
        "The time spent in resolving module cache by Libra VM"
    )
    .unwrap()
});

pub static LIBRA_MOVEVM_MODULE_CACHE_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_movevm_module_cache_seconds",
        // metric description
        "The time spent in loading module cache by Libra VM"
    )
    .unwrap()
});

pub static LIBRA_MOVEVM_LOAD_TYPE_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_movevm_load_type_seconds",
        // metric description
        "The time spent in loading types"
    )
    .unwrap()
});


pub static LIBRA_MOVEVM_VERIFY_TYPE_PARAMETERS_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_movevm_verify_type_parameters_seconds",
        // metric description
        "The time spent by Move VM to verify type arguments"
    )
    .unwrap()
});


pub static LIBRA_MOVEVM_LOAD_MODULE_CACHE_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_movevm_load_module_cache_seconds",
        // metric description
        "The time spent to load module cache"
    )
    .unwrap()
});


pub static LIBRA_MOVEVM_DATA_STORE_LOAD_MODULE_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_movevm_data_store_load_module_seconds",
        // metric description
        "The time spent to load the data store module"
    )
    .unwrap()
});

pub static LIBRA_MOVEVM_DESERIALIZE_AND_VERIFY_MODULE_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_movevm_deserialize_and_verify_module_seconds",
        // metric description
        "The time spent to deserialize and verify a module"
    )
    .unwrap()
});


pub static LIBRA_MOVEVM_LOAD_FUNCTION_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        // metric name
        "libra_movevm_load_function_seconds",
        // metric description
        "The time spent to load a function"
    )
    .unwrap()
});
