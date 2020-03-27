// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod counters;
mod pusher;

pub use crate::{
    counters::{Counter, Gauge, Metrics, PushMetricFormat},
    pusher::MetricsPusher,
};

/// Defines a list of counters with help messages
/// Additionally, requires a prefix which will be prepended to all counters
///
/// # Example of defining two counters and using the prefix libra_safety_rules for counter names
/// # Also defines a static COUNTERS which contains a reference to all the defined counters
///
/// ```ignore
/// define_counters![
///     "libra_safety_rules",
///     (
///         sign_proposal: Counter,
///         "sign_proposal counter counts sign_proposals"
///     ),
///     (
///         sign_timeout: Counter,
///         "sign_timeout counter counts sign_timeouts"
///     ),
///     (some_gauge: Gauge, "example help for a gauge metric"),
/// ];
///
/// lazy_static! {
///     pub static ref COUNTERS: Arc<Counters> = Arc::new(Counters::new());
/// }
/// ```
///
/// # Example of setting them in code
///
/// ```ignore
/// COUNTERS.some_gauge.set(42)
/// COUNTERS.some_counter.inc()
/// ```
///
#[macro_export]
macro_rules! define_counters {
    [
        $prefix:expr,$(($counter:ident: $countertype:ty, $help: expr)),* $(,)?
    ] => {
        pub struct Counters {
            $( pub $counter: $countertype, )*
        }

        impl Counters {
            pub fn new() -> Self {
                Self {
                $(
                    $counter: <$ countertype>::new(
                      format!("{}_{}", $prefix, stringify!($counter)),
                      ($help).to_string(),
                    ),
                )*
                }
            }
        }

        impl $crate::Metrics for Counters {
            fn get_metrics(&self) -> Vec<&dyn $crate::PushMetricFormat> {
                vec![$( &self.$counter, )*]
            }
        }
    };
}
