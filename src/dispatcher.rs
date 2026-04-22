use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use crate::queue::{dequeue_task, SharedQueue};
use crate::task::Task;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchPolicy {
    Fifo,
    OptimizedSjfApprox,
}

const LOOK_AHEAD: usize = 8;

pub fn dispatch_tasks(
    queue: SharedQueue,
    senders: Vec<Sender<Task>>,
    total_tasks: usize,
    policy: DispatchPolicy,
) {
    let mut dispatched = 0usize;
    let mut next_worker = 0usize;

    while dispatched < total_tasks {
        let next_task = match policy {
            DispatchPolicy::Fifo => dequeue_task(&queue),
            DispatchPolicy::OptimizedSjfApprox => pick_shortest_task(&queue),
        };

        if let Some(task) = next_task {
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

fn pick_shortest_task(queue: &SharedQueue) -> Option<Task> {
    let mut q = queue.lock().expect("queue mutex poisoned");

    if q.is_empty() {
        return None;
    }

    let take_count = LOOK_AHEAD.min(q.len());
    let mut candidates: Vec<Task> = (0..take_count)
        .filter_map(|_| q.pop_front())
        .collect();

    let shortest_idx = candidates
        .iter()
        .enumerate()
        .min_by_key(|(_, t)| t.duration_ms)
        .map(|(i, _)| i)
        .expect("candidates should not be empty");

    let chosen = candidates.remove(shortest_idx);

    for task in candidates.into_iter().rev() {
        q.push_front(task);
    }

    Some(chosen)
}