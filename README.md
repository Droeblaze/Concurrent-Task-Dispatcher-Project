# Concurrent Task Dispatcher (Rust)

> A multi-threaded task scheduling simulation built with Rust's standard library concurrency primitives.

---

## Overview

This project is a concurrent task dispatcher written in Rust. It simulates a realistic stream of incoming work where tasks arrive over time, are placed into a shared ready queue, and are assigned by a dedicated dispatcher thread to a bounded pool of worker threads — each of which executes tasks concurrently.

The goal is not to build a real operating system. It is to demonstrate core systems programming concepts through a working, measurable simulation:

- Thread design and coordination
- Shared state and ownership
- Channels for directed data flow
- Queue-based scheduling
- Bounded worker pools
- Fairness, throughput, and congestion
- Clean, panic-free shutdown
- Instrumentation and metrics collection

This project was built as a final project for a Systems Programming course at UTRGV.

---

## Features

- 500+ randomly generated tasks with mixed CPU and IO kinds
- Tasks arrive over time - not all at once
- Shared ready queue protected by `Arc<Mutex<VecDeque<Task>>>`
- Dedicated dispatcher thread with round-robin worker assignment
- Per-worker `mpsc` channels for lock-free task delivery
- FIFO + round-robin scheduling policy
- Full metrics: makespan, wait time, turnaround time, task counts
- Two selectable experiments at runtime (balanced and stressed)
- Clean shutdown with no hanging threads or infinite loops

---

## System Architecture

The pipeline flows through five stages:

```
Generator (main thread)
        ↓  [Arc<Mutex<VecDeque<Task>>>]
   Ready Queue (shared)
        ↓
   Dispatcher Thread
        ↓  [mpsc channel, one per worker]
 Worker Channels
        ↓
   Worker Threads (bounded pool)
        ↓  [Arc<Mutex<Metrics>>]
   Shared Metrics
```

### Component Breakdown

**Generator (main thread)**
The main thread creates all tasks and inserts them into the ready queue one at a time, sleeping between insertions to simulate tasks arriving over real time. Once all tasks have been generated, it waits for the dispatcher and all workers to finish.

**Ready Queue**
A `VecDeque<Task>` wrapped in `Arc<Mutex<...>>` and shared between the generator and dispatcher. Tasks are pushed to the back on arrival and popped from the front during dispatch — classic FIFO behavior. The Mutex ensures no two threads read or write simultaneously.

**Dispatcher Thread**
A single dedicated thread that continuously polls the ready queue. When a task is available, it selects the next worker using round-robin and sends the task through that worker's channel. The dispatcher never executes tasks — it only routes them. When all tasks are dispatched, it drops all sender handles and exits.

**Worker Channels**
Each worker owns the receiving end of an `mpsc::channel`. The dispatcher holds all the sender ends and cycles through them. Using per-worker channels means workers never compete over a shared lock to receive tasks — the data flow is strictly one-directional.

**Worker Threads**
Four worker threads run in parallel. Each worker waits on its channel, picks up tasks as they arrive, simulates execution via `thread::sleep`, and writes timing metrics to the shared `Metrics` struct when done. Workers exit automatically when their channel closes.

**Shared Metrics**
A `Metrics` struct wrapped in `Arc<Mutex<...>>` and shared across all workers. After completing each task, a worker locks the struct briefly to update counters and timing totals. The main thread reads the final values after all threads join.

---

## Task Model

Each task is a Rust struct with the following fields:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `u64` | Unique task identifier |
| `arrival_time` | `Instant` | When the task was created |
| `kind` | `TaskKind` | `Cpu` or `Io` |
| `duration_ms` | `u64` | Simulated execution time in milliseconds |

`TaskKind` is an enum with two variants: `Cpu` and `Io`. Both types enter the same ready queue and are processed with the same FIFO + round-robin policy. The kind is tracked separately in metrics to report CPU and IO task counts, but the current scheduler does not route them to separate queues or reserved workers. This is a known limitation and a clear candidate for future improvement.

---

## Scheduling Policy

The dispatcher uses a two-part policy:

**FIFO (First In, First Out)**
Tasks are dispatched in the order they arrived. The task at the front of the `VecDeque` is always selected next. This prevents any task from being indefinitely skipped and makes the dispatch order predictable and easy to reason about.

**Round-Robin Worker Assignment**
The dispatcher keeps an index counter and increments it modulo the number of workers for each dispatch. This distributes work evenly across the pool over time so no single worker becomes a bottleneck while others sit idle.

### Why This Policy?

- Simple to implement and easy to explain
- Fair in the sense that every task makes forward progress
- Evenly distributes assignments across workers
- No complex state or task inspection required

### Trade-offs

- FIFO creates a **convoy effect** — long tasks at the front of the queue delay shorter tasks behind them
- Round-robin does not account for current worker load — a worker that received several long tasks may fall behind while others wait idle
- CPU and IO tasks are treated identically, which misses opportunities for type-aware scheduling

A shortest-job-first or shortest-queue dispatch policy would reduce these issues but would add implementation and explanation complexity.

---

## Arrival Over Time

One of the most important design details in this system is that **tasks do not all arrive at once**.

Each task has an `arrival_time` recorded at the moment it is created. Between each task insertion, the main thread sleeps for a random interval - 5 to 30 milliseconds in the balanced experiment and 1 to 5 milliseconds in the stressed experiment. This creates a realistic trickle of incoming work over real clock time rather than dumping everything into the queue immediately.

The effect of this is visible in the program output: task arrival messages and worker execution messages are interleaved, which shows the pipeline actually running concurrently. Tasks are queuing, dispatching, and executing in parallel - not in a sequential batch.

This design choice also means wait time is a meaningful metric. A task's wait time is the gap between when it was created (arrival) and when a worker actually picked it up. If tasks arrived all at once, every task's wait time would be artificially inflated regardless of the scheduler's quality.

---

## Metrics Collected

The following metrics are printed at the end of every run:

### Required Metrics

| Metric | Description |
|--------|-------------|
| Total tasks completed | Number of tasks that ran to completion across all workers |
| Makespan | Total wall-clock time from first task creation to last task completion |
| Average wait time | Mean time a task spent in the queue between arrival and execution start |
| Average turnaround time | Mean time from task arrival to full completion (wait + execution) |

### Additional Metrics

| Metric | Description |
|--------|-------------|
| Max wait time | Worst-case wait experienced by any single task |
| CPU tasks completed | Count of `Cpu`-kind tasks that finished |
| IO tasks completed | Count of `Io`-kind tasks that finished |

**Wait time** reflects how backed up the system is. High average wait means the queue is building up faster than it is being drained.

**Turnaround time** is the total cost a task incurs — from the moment it enters the system to the moment it finishes. It combines wait time and execution time.

---

## Experiments

When the program runs, you will be prompted to select an experiment:

```
Select experiment:
  1 = Balanced workload
  2 = Stressed workload
Enter choice:
```

### Experiment A — Balanced Workload

A moderate, realistic workload designed to represent normal operating conditions.

| Parameter | Value |
|-----------|-------|
| Total tasks | 500 |
| Workers | 4 |
| Arrival gap | 5–30 ms per task |
| Duration range | 50–200 ms per task |
| CPU / IO split | ~50% / ~50% |
| Policy | FIFO + round-robin |

The balanced experiment gives the scheduler room to breathe. Arrivals are spaced out enough that the queue does not immediately flood, but task durations are long enough that workers stay busy.

### Experiment B — Stressed Workload

A high-pressure workload designed to saturate the queue and expose scheduler behavior under congestion.

| Parameter | Value |
|-----------|-------|
| Total tasks | 500 |
| Workers | 4 |
| Arrival gap | 1–5 ms per task |
| Duration range | 50–200 ms per task |
| CPU / IO split | ~50% / ~50% |
| Policy | FIFO + round-robin |

The stressed experiment compresses arrival gaps so tasks enter the queue much faster than in the balanced case. The goal is to keep constant pressure on the dispatcher and reveal how the system behaves when it can never fully catch up.

---

## Sample Results

These are representative results from two runs. Results vary slightly between runs due to randomization, but the general pattern is consistent.

### Experiment A — Balanced Workload

```
Total tasks completed:    500
Makespan:                 ~27,257 ms
Average wait time:        ~13,310.75 ms
Average turnaround time:  ~13,524.52 ms
Max wait time:            ~27,111 ms
CPU tasks completed:      236
IO tasks completed:       264
```

### Experiment B — Stressed Workload

```
Total tasks completed:    500
Makespan:                 ~27,699 ms
Average wait time:        ~13,149.97 ms
Average turnaround time:  ~13,362.96 ms
Max wait time:            ~27,531 ms
CPU tasks completed:      231
IO tasks completed:       269
```

### What These Results Suggest

The most striking finding is how similar the two experiments are. Makespans differ by less than 500 ms, and average wait times are within 200 ms of each other. This tells us that the system is already operating near saturation in the balanced case — faster arrivals in Experiment B fill the queue sooner, but the drain rate (determined by worker capacity) stays the same.

Worker processing capacity is the bottleneck, not arrival speed. With 4 workers processing tasks up to 200 ms long, the total throughput ceiling is fixed. Both experiments hit that ceiling. Future improvement would focus on increasing worker count or using smarter scheduling to reduce per-task cost.

---

## How to Build and Run

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable, 1.70 or later recommended)
- Cargo (included with Rust)

### Clone the Repository

```bash
git clone https://github.com/yourusername/concurrent-task-dispatcher.git
cd concurrent-task-dispatcher
```

> Replace `yourusername` with your actual GitHub username.

### Build

```bash
cargo build
```

### Run

```bash
cargo run
```

You will be prompted to choose an experiment (1 or 2). The program will then generate tasks, run the simulation, and print summary metrics at the end.

---

## Configuration

The primary way to select between experiments is the runtime prompt. However, if you want to adjust the underlying workload parameters, the relevant values are defined in `src/generator.rs`.

| Parameter | Where to change | Default (Balanced) | Default (Stressed) |
|-----------|----------------|--------------------|--------------------|
| Total tasks | `generator.rs` | 500 | 500 |
| Number of workers | `main.rs` | 4 | 4 |
| Arrival gap range | `generator.rs` | 5–30 ms | 1–5 ms |
| Duration range | `generator.rs` | 50–200 ms | 50–200 ms |
| CPU / IO ratio | `generator.rs` | ~50% / 50% | ~50% / 50% |

Experiments are selected at runtime, but the generation logic for each can be modified in code if you want to test a custom configuration outside the two defaults.

---

## Design Trade-offs and Limitations

### Advantages

- **Modular structure** - each component (generator, dispatcher, workers, metrics) has exactly one job, making the system easy to follow and debug
- **Safe concurrency** - `Arc`, `Mutex`, and `mpsc` channels ensure no data races; the Rust compiler enforces this at compile time
- **Fair baseline scheduler** - FIFO + round-robin prevents starvation and distributes work evenly without requiring complex state
- **Readable output** - metrics are printed in a clean summary that makes experiment comparison straightforward

### Limitations

- **FIFO does not optimize for short tasks** - a long task at the head of the queue delays all shorter tasks behind it, inflating their wait times
- **No separate CPU/IO scheduling** - both task types compete in the same queue; a two-queue design with reserved workers would allow type-aware optimization
- **Fixed worker count** - the pool does not scale dynamically; under light load, workers sit idle, and under heavy load, the queue grows without bound
- **High wait times under load** - both experiments showed average wait times over 13 seconds, which reflects near-saturation operation with the current worker count

---

## Future Improvements

The following extensions would meaningfully improve the system's fairness and throughput:

- **Separate CPU and IO queues** with dedicated or weighted worker pools
- **Reserved workers** for CPU-heavy tasks to prevent IO tasks from being starved during bursts
- **Priority scheduling** so more urgent tasks can jump ahead of less urgent ones
- **Aging** - gradually increasing the priority of tasks that have been waiting a long time, directly reducing the convoy effect
- **Work stealing** - idle workers pull tasks from busy workers' queues rather than waiting for round-robin assignment
- **Worker utilization and idle time tracking** - report how much time each worker spent actually executing versus waiting
- **Queue length statistics** - track how deep the queue gets over time to better understand congestion patterns
- **Dynamic worker scaling** - spin up new workers when the queue grows past a threshold and retire them when load drops

---

## Tool Use Disclosure

During this project, I did use **ChatGPT** as development aids - for architecture guidance, debugging help, module organization, and explanation support when writing the report.

**Example of advice I accepted:**
Both tools consistently recommended using `Arc<Mutex<VecDeque<Task>>>` for the shared ready queue and using per-worker `mpsc` channels from the dispatcher to workers instead of having workers pull directly from the queue. This was good advice as it eliminated lock contention among workers and made the shutdown mechanism clean and implicit.

**Example of advice I rejected or had to fix:**
At one point, an AI-generated code suggestion redefined the `Task` struct directly inside `main.rs` using fields like `task.name` that did not match the actual struct definition in `task.rs`. This broke the modular design and introduced type mismatch errors across multiple files. I had to discard that version, unify the struct in `task.rs`, and update all usages to use the correct field names (`arrival_time`, `kind`, `duration_ms`). The fix also required correcting the import path to `use crate::task::Task` throughout the project.

I understand all parts of the design and can explain, modify, or debug the implementation independently.

---

## Author

**Randy Cantu**
University of Texas Rio Grande Valley
Department of Computer Science

---

## License and Academic Use

This project was created for academic coursework.