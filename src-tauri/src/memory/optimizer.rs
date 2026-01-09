use super::*;
use anyhow::Result;
use sysinfo::{System, RefreshKind, MemoryRefreshKind, ProcessRefreshKind};
use std::collections::HashMap;

pub struct MemoryOptimizer {
    system: System,
}

impl MemoryOptimizer {
    pub fn new() -> Self {
        Self {
            system: System::new_with_specifics(
                RefreshKind::new()
                    .with_memory(MemoryRefreshKind::everything())
                    .with_processes(ProcessRefreshKind::everything())
            ),
        }
    }

    pub async fn get_memory_info(&self) -> Result<MemoryInfo> {
        let mut system = System::new_all();
        system.refresh_memory();

        let total = system.total_memory();
        let used = system.used_memory();
        let free = system.free_memory();
        let available = system.available_memory();

        Ok(MemoryInfo {
            total,
            used,
            free,
            available,
            cached: total.saturating_sub(used).saturating_sub(free),
            swap_total: system.total_swap(),
            swap_used: system.used_swap(),
            usage_percentage: (used as f32 / total as f32) * 100.0,
        })
    }

    pub async fn get_process_list(&self) -> Result<Vec<ProcessInfo>> {
        let mut system = System::new_all();
        system.refresh_all();

        let total_memory = system.total_memory();
        let mut processes: Vec<ProcessInfo> = system
            .processes()
            .iter()
            .map(|(pid, process)| {
                let memory_bytes = process.memory();
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string_lossy().to_string(),
                    memory_bytes,
                    memory_percentage: (memory_bytes as f32 / total_memory as f32) * 100.0,
                    cpu_percentage: process.cpu_usage(),
                    status: format!("{:?}", process.status()),
                    user: process.user_id().map(|u| format!("{:?}", u)),
                }
            })
            .collect();

        // Sort by memory usage
        processes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));

        Ok(processes)
    }

    pub async fn get_recommendations(&self) -> Result<Vec<MemoryRecommendation>> {
        let mut system = System::new_all();
        system.refresh_all();

        let mut recommendations = Vec::new();
        let total_memory = system.total_memory();

        // Find high memory processes
        let high_memory_threshold = total_memory / 10; // 10% of total memory
        
        for (pid, process) in system.processes() {
            if process.memory() > high_memory_threshold {
                recommendations.push(MemoryRecommendation {
                    category: RecommendationCategory::HighMemoryProcess,
                    title: format!("{} is using significant memory", process.name().to_string_lossy()),
                    description: format!(
                        "This process is using {} of memory ({:.1}%)",
                        format_bytes(process.memory()),
                        (process.memory() as f32 / total_memory as f32) * 100.0
                    ),
                    potential_savings: process.memory(),
                    action: RecommendedAction::Suggest,
                    process_ids: vec![pid.as_u32()],
                });
            }
        }

        // Find duplicate processes (multiple instances of same name)
        let mut process_counts: HashMap<String, Vec<(sysinfo::Pid, u64)>> = HashMap::new();
        for (pid, process) in system.processes() {
            let name = process.name().to_string_lossy().to_string();
            process_counts
                .entry(name)
                .or_default()
                .push((*pid, process.memory()));
        }

        for (name, instances) in process_counts {
            if instances.len() > 3 && !is_system_process(&name) {
                let total_mem: u64 = instances.iter().map(|(_, m)| m).sum();
                recommendations.push(MemoryRecommendation {
                    category: RecommendationCategory::DuplicateProcess,
                    title: format!("Multiple instances of {}", name),
                    description: format!(
                        "{} instances running, using {} total",
                        instances.len(),
                        format_bytes(total_mem)
                    ),
                    potential_savings: total_mem / 2, // Estimate
                    action: RecommendedAction::Suggest,
                    process_ids: instances.iter().map(|(pid, _)| pid.as_u32()).collect(),
                });
            }
        }

        // Sort by potential savings
        recommendations.sort_by(|a, b| b.potential_savings.cmp(&a.potential_savings));

        Ok(recommendations)
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn is_system_process(name: &str) -> bool {
    let system_processes = [
        "systemd", "init", "kernel", "kworker", "ksoftirqd",
        "svchost", "csrss", "wininit", "services", "lsass",
        "WindowServer", "launchd", "kernel_task",
    ];

    system_processes.iter().any(|&p| name.to_lowercase().contains(p))
}
