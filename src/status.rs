use std::time::Instant;
use sysinfo::{Disks, System};
use serde_json::{json, Value};

fn get_uptime(start: Instant) -> u64 {
    Instant::now().duration_since(start).as_secs()
}

fn get_cpu_usage(sys: &mut System) -> u64 {
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_all();
    let cpus = sys.cpus();
    let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
    let avg_usage = total_usage / cpus.len() as f32;
    avg_usage.round() as u64
}

fn get_ram(sys: &mut System) -> u64 {
    sys.refresh_memory();
    let total = sys.total_memory();
    let used = sys.used_memory();

    if total == 0 {
        return 0;
    }

    ((used as f64 / total as f64) * 100.0).round() as u64
}

fn get_disks_usage() -> u64 {
    let mut disks = Disks::new_with_refreshed_list();
    let mut total_used = 0.0;
    let mut total_space = 0.0;

    for disk in &mut disks {
        disk.refresh();
        total_space += disk.total_space() as f64;
        total_used += (disk.total_space() - disk.available_space()) as f64;
    }

    if total_space == 0.0 {
        return 0;
    }

    ((total_used / total_space) * 100.0).round() as u64
}

pub fn get_status(start: Instant) -> Value {
    let mut sys = System::new();

    json!({
        "uptime": get_uptime(start),
        "cpu": get_cpu_usage(&mut sys),
        "ram": get_ram(&mut sys),
        "disk": get_disks_usage()
    })
}