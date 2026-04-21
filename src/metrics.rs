use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::task::TaskKind;

#[derive(Debug, Default)]
pub struct Metrics {
    pub total_completed: usize,
    pub total_wait_time: Duration,
    pub total_turnaround_time: Duration,
    pub max_wait_time: Duration,
    pub cpu_completed: usize,
    pub io_completed: usize,
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

    let mut m = metrics.lock().unwrap();
    m.total_completed += 1;
    m.total_wait_time += wait_time;
    m.total_turnaround_time += turnaround_time;

    if wait_time > m.max_wait_time {
        m.max_wait_time = wait_time;
    }

    match task_kind {
        TaskKind::Cpu => m.cpu_completed += 1,
        TaskKind::Io => m.io_completed += 1,
    }
}

pub fn print_metrics(metrics: &SharedMetrics, makespan: Duration) {
    let m = metrics.lock().unwrap();

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
    println!("CPU tasks completed: {}", m.cpu_completed);
    println!("IO tasks completed: {}", m.io_completed);
}