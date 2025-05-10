use std::{
    cmp::max,
    collections::HashMap,
    fs::{read_dir, read_to_string},
    io,
};

use colored::*;
use terminal_size::{terminal_size, Width};
use nix::sys::statvfs::statvfs;

struct Drive {
    name: String,
    size: u64,
    partitions: Vec<Partition>,
}

struct Partition {
    name: String,
    size: u64,
    used: Option<u64>,
}

impl Partition {
    pub fn new(_name: String) -> Partition {
        let size = read_size(&_name).unwrap_or(0);
        let mountpoints = get_mountpoints();

        let dev_name = format!("/dev/{}", _name.split('/').last().unwrap_or(&_name));
        let used = mountpoints.get(&dev_name).and_then(|mount| {
            statvfs(mount.as_str()).ok().map(|stat| {
                let total = stat.blocks() * stat.block_size();
                let free = stat.blocks_free() * stat.block_size();
                total - free
            })
        });

        Partition {
            name: _name.clone(),
            size,
            used,
        }
    }
}

impl Drive {
    pub fn new(_name: &str) -> Drive {
        Drive {
            name: _name.to_string(),
            size: read_size(_name).unwrap_or(0),
            partitions: get_partitions(_name),
        }
    }
}

fn get_partitions(_name: &str) -> Vec<Partition> {
    read_dir(format!("/sys/block/{}/", _name))
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| (entry.path(), name.to_string()))
        })
        .filter(|(_, name)| name.starts_with(_name))
        .map(|(_, name)| Partition::new(format!("{}/{}", _name, name)))
        .collect()
}

fn read_size(name: &str) -> io::Result<u64> {
    let file = read_to_string(format!("/sys/block/{}/size", name))?;
    Ok(file.trim().parse().unwrap_or(0))
}

fn read_drives() -> Vec<Drive> {
    read_dir("/sys/block/")
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry.file_name().to_str().map(String::from).filter(|name| !name.starts_with("dm"))
        })
        .map(|name| Drive::new(&name))
        .collect()
}

fn get_mountpoints() -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(content) = read_to_string("/proc/mounts") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                map.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
    }
    map
}

fn print_drive_chart(drive: &Drive, width: usize) {
    let total_size = max(drive.size, 1);
    let mut used_width = 0;

    println!(
        "\n{} {} ({:.2} GB)",
        "Drive:".bold().blue(),
        drive.name.bold(),
        drive.size as f64 * 512.0 / 1024f64.powi(3)
    );
    print!("[");
    
    let symbols = ["█", "▓", "▒", "░"];
    let colors = [Color::Green, Color::Yellow, Color::Blue, Color::Magenta, Color::Cyan];

    for (i, partition) in drive.partitions.iter().enumerate() {
        let part_ratio = partition.size as f64 / total_size as f64;
        let part_width = ((part_ratio * width as f64).round() as usize).min(width - used_width);
        if part_width == 0 {
            continue;
        }

        let symbol = symbols[i % symbols.len()];
        let color = colors[i % colors.len()];
        let visual = symbol.repeat(part_width);
        print!("{}", visual.color(color));
        used_width += part_width;
    }

    if used_width < width {
        print!("{}", " ".repeat(width - used_width));
    }

    println!("]");

    let name_width = drive
        .partitions
        .iter()
        .map(|p| p.name.len())
        .max()
        .unwrap_or(0);
    let chart_width = 20;
    let size_text_width = 18;

    for (i, partition) in drive.partitions.iter().enumerate() {
        let color = colors[i % colors.len()];
        let size_gb = partition.size as f64 * 512.0 / 1024f64.powi(3);
        let used_gb = partition.used.map(|u| u as f64 / 1024f64.powi(3)).unwrap_or(0.0);

        let name_str = format!("{:width$}", partition.name, width = name_width);
        let size_str = format!("{:.1} / {:.1} GB", used_gb, size_gb);
        let size_str = format!("{:>width$}", size_str, width = size_text_width);

        let usage_bar = if let Some(u) = partition.used {
            let total_bytes = partition.size * 512;
            let ratio = (u as f64 / total_bytes as f64).clamp(0.0, 1.0);
            let filled = (ratio * chart_width as f64).round() as usize;
            let bar = "█".repeat(filled) + &"░".repeat(chart_width - filled);
            format!("{}", bar.color(color))
        } else {
            format!("{}", "Unmounted".dimmed())
        };

        let mountpoints = get_mountpoints();
        let dev_path = format!("/dev/{}", partition.name.split('/').last().unwrap_or(&partition.name));
        let mountpoint = mountpoints
            .get(&dev_path)
            .cloned()
            .unwrap_or_else(|| "-".to_string());

        println!(
            "  {} {} {} {} {}",
            "■".color(color),
            name_str,
            usage_bar,
            size_str,
            mountpoint
        );
    }
}

fn get_terminal_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w.saturating_sub(10).min(100) as usize 
    } else {
        80
    }
}

fn main() {
    let drives: Vec<Drive> = read_drives();
    let chart_width = get_terminal_width();

    for drive in &drives {
        print_drive_chart(drive, chart_width);
    }
}

