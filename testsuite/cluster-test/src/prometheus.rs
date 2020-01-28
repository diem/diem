// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::{bail, format_err, Result};
use reqwest::Url;
use serde::Deserialize;
use std::{collections::HashMap, time::Duration};

#[derive(Clone)]
pub struct Prometheus {
    url: Url,
    client: reqwest::blocking::Client,
    public_url: Url,
}

pub struct MatrixResponse {
    inner: HashMap<String, TimeSeries>,
}

pub struct TimeSeries {
    inner: Vec<(u64, f64)>,
}

impl Prometheus {
    pub fn new(ip: &str, workspace: &str) -> Self {
        let url = format!("http://{}:9091", ip)
            .parse()
            .expect("Failed to parse prometheus url");
        let public_url = format!("http://prometheus.{}.aws.hlw3truzy4ls.com:9091", workspace)
            .parse()
            .expect("Failed to parse prometheus public url");
        let client = reqwest::blocking::Client::new();
        Self {
            url,
            client,
            public_url,
        }
    }

    pub fn link_to_dashboard(&self, start: Duration, end: Duration) -> String {
        format!(
            "{}d/overview10/overview?orgId=1&from={}&to={}",
            self.public_url,
            start.as_millis(),
            end.as_millis()
        )
    }

    fn query_range(
        &self,
        query: String,
        start: &Duration,
        end: &Duration,
        step: u64,
    ) -> Result<MatrixResponse> {
        let url = self
            .url
            .join(&format!(
                "api/datasources/proxy/1/api/v1/query_range?query={}&start={}&end={}&step={}",
                query,
                start.as_secs(),
                end.as_secs(),
                step
            ))
            .expect("Failed to make query_range url");

        let response = self
            .client
            .get(url.clone())
            .send()
            .map_err(|e| format_err!("Failed to query prometheus: {:?}", e))?;

        // We don't check HTTP error code here
        // Prometheus supplies error status in json response along with error text

        let response: PrometheusResponse = response.json().map_err(|e| {
            format_err!("Failed to parse prometheus response: {:?}. Url: {}", e, url)
        })?;

        match response.data {
            Some(data) => MatrixResponse::from_prometheus(data),
            None => bail!(
                "Prometheus query failed: {} {}",
                response.error_type,
                response.error
            ),
        }
    }
    pub fn query_range_avg(
        &self,
        query: String,
        start: &Duration,
        end: &Duration,
        step: u64,
    ) -> Result<f64> {
        let response = self.query_range(query, start, end, step)?;
        response
            .avg()
            .ok_or_else(|| format_err!("Failed to compute avg"))
    }
}

impl MatrixResponse {
    pub fn avg(&self) -> Option<f64> {
        if self.inner.is_empty() {
            return None;
        }
        let mut sum = 0.;
        let mut count = 0usize;
        for time_series in self.inner.values() {
            if let Some(ts_avg) = time_series.avg() {
                sum += ts_avg;
                count += 1;
            }
        }
        if count == 0 {
            None
        } else {
            Some(sum / (count as f64))
        }
    }
}

impl TimeSeries {
    pub fn get(&self) -> &[(u64, f64)] {
        &self.inner
    }

    pub fn avg(&self) -> Option<f64> {
        let mut sum = 0.;
        let mut count = 0usize;
        for (_, v) in self.inner.iter() {
            if !v.is_normal() {
                // Some time series can return NaN (for example, latency query that has division in
                // it). If we include this NaN in sum, it will 'poison' it - if one
                // of values is NaN, sum will be NaN too, and avg will be NaN
                // Instead of poisoning, we simply ignore NaN values when calculating avg
                continue;
            }
            sum += *v;
            count += 1;
        }
        if count == 0 {
            None
        } else {
            Some(sum / (count as f64))
        }
    }
}

#[derive(Debug, Deserialize)]
struct PrometheusResponse {
    data: Option<PrometheusData>,
    #[serde(default)]
    error: String,
    #[serde(alias = "errorType")]
    #[serde(default)]
    error_type: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct PrometheusData {
    result: Vec<PrometheusResult>,
}

#[derive(Debug, Deserialize)]
struct PrometheusResult {
    metric: PrometheusMetric,
    values: Vec<(u64, String)>,
}

#[derive(Debug, Deserialize)]
struct PrometheusMetric {
    op: Option<String>,
    peer_id: String,
}

impl MatrixResponse {
    fn from_prometheus(data: PrometheusData) -> Result<Self> {
        let mut inner = HashMap::new();
        for entry in data.result {
            let peer_id = entry.metric.peer_id;
            if entry.values.is_empty() {
                continue;
            }
            let time_series = TimeSeries::from_prometheus(entry.values)?;
            inner.insert(peer_id, time_series);
        }
        Ok(Self { inner })
    }
}

impl TimeSeries {
    fn from_prometheus(values: Vec<(u64, String)>) -> Result<Self> {
        let mut inner = vec![];
        for (ts, value) in values {
            let value = value.parse().map_err(|e| {
                format_err!("Failed to parse entry in prometheus time series: {:?}", e)
            })?;
            inner.push((ts, value));
        }
        Ok(TimeSeries { inner })
    }
}
