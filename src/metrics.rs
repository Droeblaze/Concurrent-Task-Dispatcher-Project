use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::task::TaskKind;

#[derive(Debug, Default)]
pub struct Metrics {
    pub total_completed: usize,
    pub total_wait_time: Duration,
    pub total_turnaround_time: Duration,
    pub max_wait_time: Duration,
    pub cpu_tasks: usize,
    pub io_tasks: usize,
}

pub type SharedMetrics = Arc<Mutex<Metrics>>;

pub fn create_metrics() -> SharedMetrics {
    Arc::new(Mutex::new(Metrics::default()))
}

pub fn record_task_completion(
    metrics: &SharedMetrics,
    task_kind: TaskKind,
    created_at: Instant,
    started_at: Instant,
    finished_at: Instant,
) {
    let wait_time = started_at.duration_since(created_at);
    let turnaround_time = finished_at.duration_since(created_at);

    let mut m = metrics.lock().expect("metrics mutex poisoned");
    m.total_completed += 1;
    m.total_wait_time += wait_time;
    m.total_turnaround_time += turnaround_time;

    if wait_time > m.max_wait_time {
        m.max_wait_time = wait_time;
    }

    match task_kind {
        TaskKind::Cpu => m.cpu_tasks += 1,
        TaskKind::Io => m.io_tasks += 1,
    }
}

pub fn print_metrics(metrics: &SharedMetrics, makespan: Duration) {
    let m = metrics.lock().expect("metrics mutex poisoned");

    let avg_wait = if m.total_completed > 0 {
        m.total_wait_time.as_millis() as f64 / m.total_completed as f64
    } else {
        0.0
    };

    let avg_turnaround = if m.total_completed > 0 {
        m.total_turnaround_time.as_millis() as f64 / m.total_completed as f64
    } else {
        0.0
    };

    println!();
    println!("===== Summary Metrics =====");
    println!("Total tasks completed: {}", m.total_completed);
    println!("Makespan: {} ms", makespan.as_millis());
    println!("Average wait time: {:.2} ms", avg_wait);
    println!("Average turnaround time: {:.2} ms", avg_turnaround);
    println!("Max wait time: {} ms", m.max_wait_time.as_millis());
    println!("CPU tasks completed: {}", m.cpu_tasks);
    println!("IO tasks completed: {}", m.io_tasks);
}

pub fn write_metrics_to_file(
    metrics: &SharedMetrics,
    makespan: Duration,
    filename: &str,
) {
    let m = metrics.lock().expect("metrics mutex poisoned");

    let avg_wait = if m.total_completed > 0 {
        m.total_wait_time.as_millis() as f64 / m.total_completed as f64
    } else {
        0.0
    };

    let avg_turnaround = if m.total_completed > 0 {
        m.total_turnaround_time.as_millis() as f64 / m.total_completed as f64
    } else {
        0.0
    };

    let mut file = File::create(filename).expect("failed to create metrics file");

    writeln!(file, "===== Summary Metrics =====").unwrap();
    writeln!(file, "Total tasks completed: {}", m.total_completed).unwrap();
    writeln!(file, "Makespan: {} ms", makespan.as_millis()).unwrap();
    writeln!(file, "Average wait time: {:.2} ms", avg_wait).unwrap();
    writeln!(file, "Average turnaround time: {:.2} ms", avg_turnaround).unwrap();
    writeln!(file, "Max wait time: {} ms", m.max_wait_time.as_millis()).unwrap();
    writeln!(file, "CPU tasks completed: {}", m.cpu_tasks).unwrap();
    writeln!(file, "IO tasks completed: {}", m.io_tasks).unwrap();

    println!("Metrics written to {}", filename);
}