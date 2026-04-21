use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::task::Task;

pub type SharedQueue = Arc<Mutex<VecDeque<Task>>>;

pub fn create_queue() -> SharedQueue {
    Arc::new(Mutex::new(VecDeque::new()))
}

pub fn enqueue_task(queue: &SharedQueue, task: Task) {
    let mut q = queue.lock().expect("queue mutex poisoned");
    q.push_back(task);
}

pub fn dequeue_task(queue: &SharedQueue) -> Option<Task> {
    let mut q = queue.lock().expect("queue mutex poisoned");
    q.pop_front()
}

pub fn queue_len(queue: &SharedQueue) -> usize {
    let q = queue.lock().expect("queue mutex poisoned");
    q.len()
}