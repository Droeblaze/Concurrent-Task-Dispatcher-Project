use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::metrics::{record_task_completion, SharedMetrics};
use crate::task::Task;

pub fn start_workers(
    receivers: Vec<Receiver<Task>>,
    metrics: SharedMetrics,
) -> Vec<JoinHandle<()>> {
    receivers
        .into_iter()
        .enumerate()
        .map(|(worker_id, rx)| {
            let metrics = metrics.clone();

            thread::spawn(move || {
                while let Ok(task) = rx.recv() {
                    let started_at = Instant::now();

                    println!(
                        "Worker {} started task {} [{:?}]",
                        worker_id, task.id, task.kind
                    );

                    thread::sleep(Duration::from_millis(task.duration_ms));

                    let finished_at = Instant::now();

                    println!(
                        "Worker {} finished task {} [{:?}]",
                        worker_id, task.id, task.kind
                    );

                    record_task_completion(
                        &metrics,
                        task.kind,
                        task.created_at,
                        started_at,
                        finished_at,
                    );
                }

                println!("Worker {} shutting down.", worker_id);
            })
        })
        .collect()
}