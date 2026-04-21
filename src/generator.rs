use rand::Rng;
use crate::task::{Task, TaskKind};

pub fn generate_tasks(total: usize, stressed: bool) -> Vec<Task> {
    let mut rng = rand::thread_rng();
    let mut tasks = Vec::new();
    let mut current_arrival_time = 0_u64;

    for i in 1..=total {
        let arrival_gap = if stressed {
            rng.gen_range(1..6)
        } else {
            rng.gen_range(5..31)
        };

        current_arrival_time += arrival_gap;

        let kind = if rng.gen_bool(0.5) {
            TaskKind::Cpu
        } else {
            TaskKind::Io
        };

        let duration_ms = rng.gen_range(50..201);

        tasks.push(Task::new(
            i as u32,
            current_arrival_time,
            kind,
            duration_ms,
        ));
    }

    tasks
}