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

use dispatcher::dispatch_tasks;
use generator::generate_tasks;
use metrics::{create_metrics, print_metrics};
use queue::{create_queue, enqueue_task};
use worker::start_workers;

fn main() {
    println!("Select experiment:");
    println!("1 = Balanced workload");
    println!("2 = Stressed workload");

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let stressed = match input.trim() {
        "2" => {
            println!("Running Experiment B: Stressed workload");
            true
        }
        _ => {
            println!("Running Experiment A: Balanced workload");
            false
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

    let dispatcher_handle = thread::spawn(move || {
        dispatch_tasks(dispatcher_queue, dispatcher_senders, total_tasks);
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

    dispatcher_handle.join().unwrap();

    for handle in worker_handles {
        handle.join().unwrap();
    }

    let makespan = simulation_start.elapsed();

    println!("All workers shut down cleanly.");
    print_metrics(&metrics, makespan);
}