use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskKind {
    Cpu,
    Io,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: u32,
    pub arrival_time: u64,
    pub kind: TaskKind,
    pub duration_ms: u64,
    pub created_at: Instant,
}

impl Task {
    pub fn new(id: u32, arrival_time: u64, kind: TaskKind, duration_ms: u64) -> Self {
        Self {
            id,
            arrival_time,
            kind,
            duration_ms,
            created_at: Instant::now(),
        }
    }
}