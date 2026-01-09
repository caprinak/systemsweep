// src-tauri/src/system/mod.rs
use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt, CpuExt, DiskExt, ProcessExt, Disks};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,
    pub total_memory: u64,
    pub used_memory: u64,
    pub cpu_count: usize,
    pub cpu_brand: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub usage_percent: f32,
    pub file_system: String,
    pub is_removable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub swap_free: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory: u64,
    pub cpu_usage: f32,
    pub status: String,
}

pub fn get_system_info() -> SystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    SystemInfo {
        os_name: sys.name().unwrap_or_default(),
        os_version: sys.os_version().unwrap_or_default(),
        kernel_version: sys.kernel_version().unwrap_or_default(),
        hostname: sys.host_name().unwrap_or_default(),
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        cpu_count: sys.cpus().len(),
        cpu_brand: sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default(),
    }
}

pub fn get_disk_usage() -> Vec<DiskInfo> {
    let mut sys = System::new_all();
    sys.refresh_disks();
    
    sys.disks().iter().map(|disk| {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total.saturating_sub(available);
        let usage_percent = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        
        DiskInfo {
            name: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            total_space: total,
            available_space: available,
            used_space: used,
            usage_percent,
            file_system: disk.file_system().to_string_lossy().to_string(),
            is_removable: disk.is_removable(),
        }
    }).collect()
}

pub fn get_memory_usage() -> MemoryInfo {
    let mut sys = System::new_all();
    sys.refresh_memory();
    
    let total = sys.total_memory();
    let used = sys.used_memory();
    let available = sys.available_memory();
    let free = total.saturating_sub(used);
    
    MemoryInfo {
        total,
        used,
        free,
        available,
        swap_total: sys.total_swap(),
        swap_used: sys.used_swap(),
        swap_free: sys.free_swap(),
        usage_percent: if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        },
    }
}

pub fn get_running_processes() -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let mut processes: Vec<ProcessInfo> = sys.processes()
        .iter()
        .map(|(pid, process)| ProcessInfo {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().to_string(),
            memory: process.memory(),
            cpu_usage: process.cpu_usage(),
            status: format!("{:?}", process.status()),
        })
        .collect();
    
    processes.sort_by(|a, b| b.memory.cmp(&a.memory));
    processes.truncate(100);
    processes
}
