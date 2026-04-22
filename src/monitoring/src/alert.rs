//! 告警管理

pub struct AlertManager {
    rules: Vec<AlertRule>,
}

#[derive(Debug, Clone)]
pub struct AlertRule {
    pub name: String,
    pub condition: String,
    pub severity: AlertSeverity,
}

#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl AlertManager {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }

    pub fn check(&self) -> Vec<String> {
        // TODO: 实现告警检查
        vec![]
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}
