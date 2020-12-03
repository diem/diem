// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use prometheus::{
    proto::{LabelPair, Metric, MetricFamily, MetricType},
    Encoder, Result,
};
use std::{collections::HashMap, io::Write};

const JSON_FORMAT: &str = "application/json";

/// An implementation of an [`Encoder`](::Encoder) that converts a `MetricFamily` proto message
/// into `fbagent` json
///
/// This implementation converts metric{dimensions,...} -> value to a flat string with a value.
/// e.g., "requests{method="GET", service="accounts"} -> 8 into
/// requests.GET.account -> 8
/// For now, it ignores timestamps (if set on the metric)
#[derive(Debug, Default)]
pub struct JsonEncoder;

impl Encoder for JsonEncoder {
    fn encode<W: Write>(&self, metric_familys: &[MetricFamily], writer: &mut W) -> Result<()> {
        let mut export_me: HashMap<String, f64> = HashMap::new();

        for mf in metric_familys {
            let name = mf.get_name();
            let metric_type = mf.get_field_type();

            for m in mf.get_metric() {
                match metric_type {
                    MetricType::COUNTER => {
                        export_me.insert(
                            flatten_metric_with_labels(name, m),
                            m.get_counter().get_value(),
                        );
                    }
                    MetricType::GAUGE => {
                        export_me.insert(
                            flatten_metric_with_labels(name, m),
                            m.get_gauge().get_value(),
                        );
                    }
                    MetricType::HISTOGRAM => {
                        // write the sum and counts
                        let h = m.get_histogram();
                        export_me.insert(
                            flatten_metric_with_labels(&format!("{}_count", name), m),
                            h.get_sample_count() as f64,
                        );
                        export_me.insert(
                            flatten_metric_with_labels(&format!("{}_sum", name), m),
                            h.get_sample_sum(),
                        );
                    }
                    _ => {
                        // do nothing; unimplemented
                    }
                }
            }
        }

        writer.write_all(serde_json::to_string(&export_me).unwrap().as_bytes())?;
        Ok(())
    }

    fn format_type(&self) -> &str {
        JSON_FORMAT
    }
}

/**
This method takes Prometheus metrics with dimensions (represented as label:value tags)
and converts it into a dot-separated string.

Example:
Prometheus metric: error_count{method: "get_account", error="connection_error"}
Result: error_count.get_account.connection_error

If the set of labels is empty, only the name is returned
Example:
Prometheus metric: errors
Result: errors

This is useful when exporting metric data to flat time series.
*/
fn flatten_metric_with_labels(name: &str, metric: &Metric) -> String {
    let res = String::from(name);

    if metric.get_label().is_empty() {
        res
    } else {
        // string-list.join(".")
        let values: Vec<&str> = metric
            .get_label()
            .iter()
            .map(LabelPair::get_value)
            .filter(|&x| !x.is_empty())
            .collect();
        let values = values.join(".");
        if !values.is_empty() {
            format!("{}.{}", res, values)
        } else {
            res
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::{
        core::{Collector, Metric},
        IntCounter, IntCounterVec, Opts,
    };
    use serde_json::Value;

    #[test]
    fn test_flatten_labels() {
        // generate counters for testing
        let counter = IntCounter::new("counter_1", "Test counter 1").unwrap();
        let res = flatten_metric_with_labels("counter_1", &counter.metric());
        assert_eq!("counter_1", res.as_str());

        let counter = IntCounterVec::new(
            Opts::new("counter_2", "Example counter for testing"),
            &["label_me"],
        )
        .unwrap();
        let res =
            flatten_metric_with_labels("counter_2", &counter.with_label_values(&[""]).metric());
        assert_eq!("counter_2", res.as_str());

        let res = flatten_metric_with_labels(
            "counter_2",
            &counter.with_label_values(&["hello"]).metric(),
        );
        assert_eq!("counter_2.hello", res.as_str());

        let counter = IntCounterVec::new(
            Opts::new("counter_2", "Example counter for testing"),
            &["label_me", "label_me_too"],
        )
        .unwrap();
        let res =
            flatten_metric_with_labels("counter_3", &counter.with_label_values(&["", ""]).metric());
        assert_eq!("counter_3", res.as_str());

        let res = flatten_metric_with_labels(
            "counter_3",
            &counter.with_label_values(&["hello", "world"]).metric(),
        );
        assert_eq!("counter_3.hello.world", res.as_str());
    }

    #[test]
    fn test_encoder() {
        let counter = IntCounterVec::new(
            Opts::new("testing_count", "Test Counter"),
            &["method", "result"],
        )
        .unwrap();
        // add some test data
        counter.with_label_values(&["get", "302"]).inc();
        counter.with_label_values(&["get", "302"]).inc();
        counter.with_label_values(&["get", "404"]).inc();
        counter.with_label_values(&["put", ""]).inc();

        let metric_family = counter.collect();
        let mut data_writer = Vec::<u8>::new();
        let encoder = JsonEncoder;
        let res = encoder.encode(&metric_family, &mut data_writer);
        assert!(res.is_ok());

        let expected: &str = r#"
        {
            "testing_count.get.302": 2.0,
            "testing_count.get.404": 1.0,
            "testing_count.put": 1.0
        }"#;

        let v: Value = serde_json::from_slice(&data_writer).unwrap();
        let expected_v: Value = serde_json::from_str(expected).unwrap();

        assert_eq!(v, expected_v);
    }
}
