//! 指标收集

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub labels: std::collections::HashMap<String, String>,
}

pub struct MetricsCollector {
    metrics: Vec<Metric>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
        }
    }

    pub fn record(&mut self, name: &str, value: f64) {
        self.metrics.push(Metric {
            name: name.to_string(),
            value,
            labels: std::collections::HashMap::new(),
        });
    }

    pub fn export(&self) -> String {
        let mut output = String::new();
        for metric in &self.metrics {
            output.push_str(&format!("{} {}\n", metric.name, metric.value));
        }
        output
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
