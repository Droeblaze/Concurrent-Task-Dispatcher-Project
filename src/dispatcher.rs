use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use crate::queue::{dequeue_task, SharedQueue};
use crate::task::Task;

pub fn dispatch_tasks(
    queue: SharedQueue,
    senders: Vec<Sender<Task>>,
    total_tasks: usize,
) {
    let mut dispatched = 0usize;
    let mut next_worker = 0usize;

    while dispatched < total_tasks {
        if let Some(task) = dequeue_task(&queue) {
            let worker_index = next_worker % senders.len();

            senders[worker_index]
                .send(task)
                .expect("failed to send task to worker");

            next_worker += 1;
            dispatched += 1;
        } else {
            thread::sleep(Duration::from_millis(5));
        }
    }
}