use anyhow::Result;
use sentinel_common::{Action, IptablesRule, Task, TaskType};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct IptablesManager {
    // Track applied rules for rollback purposes
    applied_rules: Arc<Mutex<Vec<String>>>,
}

impl IptablesManager {
    pub fn new() -> Self {
        Self {
            applied_rules: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Process iptables task from server
    pub async fn process_task(&self, task: &Task) -> Result<()> {
        match task.task_type {
            TaskType::UpdateIptables => {
                if let Ok(rules) = serde_json::from_value::<Vec<IptablesRule>>(task.payload.clone()) {
                    tracing::info!("Processing {} iptables rules from task {}", rules.len(), task.id);

                    for rule in rules {
                        if let Err(e) = self.apply_rule(&rule).await {
                            tracing::error!("Failed to apply iptables rule: {}", e);
                            // Continue with other rules even if one fails
                        }
                    }
                } else if let Ok(rule) = serde_json::from_value::<IptablesRule>(task.payload.clone()) {
                    tracing::info!("Processing single iptables rule from task {}", task.id);
                    self.apply_rule(&rule).await?;
                } else {
                    anyhow::bail!("Invalid iptables task payload format");
                }
            }
            _ => {
                anyhow::bail!("Invalid task type for iptables manager: {:?}", task.task_type);
            }
        }

        Ok(())
    }

    pub async fn apply_rule(&self, rule: &IptablesRule) -> Result<()> {
        tracing::info!("Applying iptables rule: {:?}", rule);

        // Check if we have permission to execute iptables
        if !self.check_iptables_permission().await? {
            anyhow::bail!("Insufficient permissions to execute iptables commands");
        }

        let mut cmd = Command::new("iptables");

        match &rule.action {
            Action::Insert => cmd.arg("-I"),
            Action::Append => cmd.arg("-A"),
            Action::Delete => cmd.arg("-D"),
        };

        cmd.arg(&rule.chain);

        if let Some(protocol) = &rule.protocol {
            cmd.args(["-p", protocol]);
        }

        if let Some(source) = &rule.source {
            cmd.args(["-s", source]);
        }

        if let Some(destination) = &rule.destination {
            cmd.args(["-d", destination]);
        }

        if let Some(dport) = &rule.dport {
            cmd.args(["--dport", &dport.to_string()]);
        }

        if let Some(sport) = &rule.sport {
            cmd.args(["--sport", &sport.to_string()]);
        }

        cmd.args(["-j", &rule.target]);

        let output = cmd.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("iptables command failed: {}", stderr);
            anyhow::bail!("iptables command failed: {}", stderr);
        }

        // Record applied rule for tracking
        let rule_description = format!("{:?}", rule);
        self.applied_rules.lock().await.push(rule_description);

        tracing::info!("iptables rule applied successfully");
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn list_rules(&self, chain: &str) -> Result<Vec<String>> {
        tracing::info!("Listing iptables rules for chain: {}", chain);

        let output = Command::new("iptables")
            .args(["-L", chain, "-n", "--line-numbers"])
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to list iptables rules: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .skip(2) // Skip header lines
            .map(String::from)
            .collect())
    }

    #[allow(dead_code)]
    pub async fn save_rules(&self) -> Result<String> {
        tracing::info!("Saving iptables rules");

        let output = Command::new("iptables-save").output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to save iptables rules: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    #[allow(dead_code)]
    pub async fn restore_rules(&self, rules: &str) -> Result<()> {
        tracing::info!("Restoring iptables rules");

        let mut cmd = Command::new("iptables-restore")
            .stdin(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = cmd.stdin.take() {
            use std::io::Write;
            stdin.write_all(rules.as_bytes())?;
        }

        let status = cmd.wait()?;
        if !status.success() {
            anyhow::bail!("Failed to restore iptables rules");
        }

        tracing::info!("iptables rules restored successfully");
        Ok(())
    }

    /// Check if we have permission to execute iptables commands
    async fn check_iptables_permission(&self) -> Result<bool> {
        // Try to list rules in a safe way to check permissions
        let output = Command::new("iptables")
            .arg("-L")
            .arg("-n")
            .arg("--line-numbers")
            .output()?;

        Ok(output.status.success())
    }

    /// Get list of applied rules for monitoring/debugging
    #[allow(dead_code)]
    pub async fn get_applied_rules(&self) -> Vec<String> {
        self.applied_rules.lock().await.clone()
    }

    /// Clear applied rules history
    #[allow(dead_code)]
    pub async fn clear_applied_rules_history(&self) {
        self.applied_rules.lock().await.clear();
    }
}