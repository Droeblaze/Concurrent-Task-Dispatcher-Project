mod dispatcher;
mod generator;
mod metrics;
mod queue;
mod task;
mod worker;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use dispatcher::{dispatch_tasks, DispatchPolicy};
use generator::generate_tasks;
use metrics::{create_metrics, print_metrics, write_metrics_to_file};
use queue::{create_queue, enqueue_task};
use worker::start_workers;

fn main() {
    println!("Select workload:");
    println!("1 = Balanced workload");
    println!("2 = Stressed workload");

    let mut workload_input = String::new();
    io::stdin()
        .read_line(&mut workload_input)
        .expect("failed to read workload input");

    let stressed = match workload_input.trim() {
        "2" => {
            println!("Running stressed workload.");
            true
        }
        _ => {
            println!("Running balanced workload.");
            false
        }
    };

    println!("\nSelect scheduling policy:");
    println!("1 = FIFO");
    println!("2 = Optimized (Shortest Job First approximation)");

    let mut policy_input = String::new();
    io::stdin()
        .read_line(&mut policy_input)
        .expect("failed to read policy input");

    let policy = match policy_input.trim() {
        "2" => {
            println!("Using optimized scheduling policy.");
            DispatchPolicy::OptimizedSjfApprox
        }
        _ => {
            println!("Using FIFO scheduling policy.");
            DispatchPolicy::Fifo
        }
    };

    println!("--------------------------------------");

    let num_workers = 4;
    let tasks = generate_tasks(500, stressed);
    let total_tasks = tasks.len();
    let queue = create_queue();
    let metrics = create_metrics();
    let mut senders = Vec::new();
    let mut receivers = Vec::new();

    for _ in 0..num_workers {
        let (tx, rx) = mpsc::channel();
        senders.push(tx);
        receivers.push(rx);
    }

    let simulation_start = Instant::now();
    let worker_handles = start_workers(receivers, metrics.clone());
    let dispatcher_queue = queue.clone();
    let dispatcher_senders = senders;
    let dispatcher_policy = policy;

    let dispatcher_handle = thread::spawn(move || {
        dispatch_tasks(
            dispatcher_queue,
            dispatcher_senders,
            total_tasks,
            dispatcher_policy,
        );
    });

    let mut previous_arrival = 0_u64;

    for task in tasks {
        let gap = task.arrival_time.saturating_sub(previous_arrival);

        if gap > 0 {
            thread::sleep(Duration::from_millis(gap));
        }

        println!("Task {} arrived and was enqueued", task.id);
        enqueue_task(&queue, task);

        previous_arrival += gap;
    }

    println!("All tasks enqueued.");

    dispatcher_handle.join().expect("dispatcher thread panicked");

    for handle in worker_handles {
        handle.join().expect("worker thread panicked");
    }

    let makespan = simulation_start.elapsed();

    println!("\nAll workers shut down cleanly.");
    print_metrics(&metrics, makespan);

    let policy_str = match policy {
        DispatchPolicy::Fifo => "fifo",
        DispatchPolicy::OptimizedSjfApprox => "optimized",
    };

    let workload_str = if stressed {
        "stressed"
    } else {
        "balanced"
    };

    let filename = format!("{}_{}_output.txt", policy_str, workload_str);

    write_metrics_to_file(&metrics, makespan, &filename);

    println!("Saved experiment results to {}", filename);
}