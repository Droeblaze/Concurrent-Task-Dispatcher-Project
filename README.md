# Concurrent Task Dispatcher (Rust)

> A multi-threaded task scheduling simulation built with Rust's standard library concurrency primitives.

---

## Overview

This project is a concurrent task dispatcher written in Rust. It simulates a realistic stream of incoming work where tasks arrive over time, are placed into a shared ready queue, and are assigned by a dedicated dispatcher thread to a bounded pool of worker threads that execute them concurrently.

The project demonstrates core systems programming concepts:

- Thread design and coordination
- Shared state and ownership
- Channels for directed data flow
- Queue-based scheduling
- Bounded worker pools
- Fairness, throughput, and congestion
- Clean, panic-free shutdown
- Instrumentation and metrics collection

Two scheduling policies are implemented and compared: a baseline **FIFO** policy and an **optimized Shortest Job First (SJF) approximation**. Both are selectable at runtime, and both are tested across two workload configurations (balanced and stressed). This is a simulation, not a real operating system.

---

## Features

- 500 randomly generated tasks with mixed CPU and IO kinds
- Reproducible results via a fixed random seed (seed 42)
- Tasks arrive over time, not all at once
- Shared ready queue protected by `Arc<Mutex<VecDeque<Task>>>`
- Dedicated dispatcher thread with selectable scheduling policy
- Per-worker `mpsc` channels for lock-free task delivery
- Round-robin worker assignment under both policies
- Full metrics: makespan, wait time, turnaround time, task counts
- Experiment output automatically saved to labeled `.txt` files
- Clean shutdown with no hanging threads or panics

---

## System Architecture

The pipeline flows through five stages:

```
Generator (main thread)
        |  [Arc<Mutex<VecDeque<Task>>>]
   Ready Queue (shared)
        |
   Dispatcher Thread  [DispatchPolicy: Fifo | OptimizedSjfApprox]
        |  [mpsc channel, one per worker]
 Worker Channels
        |
   Worker Threads (bounded pool of 4)
        |  [Arc<Mutex<Metrics>>]
   Shared Metrics
```

### Component Breakdown

**Generator (main thread)**
Creates all 500 tasks and inserts them into the ready queue one at a time, sleeping between insertions to simulate tasks arriving over real time. After all tasks are enqueued, it waits for the dispatcher and workers to finish.

**Ready Queue**
A `VecDeque<Task>` wrapped in `Arc<Mutex<...>>` shared between the generator and dispatcher. Tasks go in the back on arrival and come out according to the active scheduling policy.

**Dispatcher Thread**
A single dedicated thread that polls the ready queue and routes tasks to workers. The routing logic depends on the active `DispatchPolicy`. Under `Fifo`, it takes the front task. Under `OptimizedSjfApprox`, it scans up to 8 tasks and dispatches the shortest one. Worker assignment always uses round-robin.

**Worker Channels**
Each of the 4 workers owns the receiving end of an `mpsc::channel`. The dispatcher holds all sender ends. Workers never compete over a shared lock to receive tasks.

**Worker Threads**
Four workers run in parallel. Each waits on its channel, executes arriving tasks via `thread::sleep`, and writes timing data to the shared `Metrics` struct when done. Workers exit automatically when their channel closes.

**Shared Metrics**
An `Arc<Mutex<Metrics>>` struct shared across all workers. After each task, a worker briefly locks it to update counters and timing totals. Results are printed and saved to a file at the end of the run.

---

## Task Model

Each task is a Rust struct with the following fields:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `u32` | Unique task identifier |
| `arrival_time` | `u64` | Simulated arrival time in ms |
| `kind` | `TaskKind` | `Cpu` or `Io` |
| `duration_ms` | `u64` | Simulated execution time in ms |
| `created_at` | `Instant` | Wall-clock creation time for metric calculations |

`TaskKind` is an enum with two variants: `Cpu` and `Io`. Both types enter the same ready queue and are dispatched by the same policy. Kind is tracked separately in metrics but is not currently used to route tasks to different queues or reserved workers. This is a known limitation and a clear candidate for future improvement.

---

## Scheduling Policies

### FIFO (Baseline)

Tasks are dispatched in the order they arrived. The task at the front of the queue is always selected next. No task is ever skipped, which prevents starvation. Worker assignment uses round-robin.

**Trade-offs:** Long tasks block shorter ones behind them (convoy effect). Round-robin does not account for current worker load.

### Optimized SJF Approximation

Instead of blindly taking the front task, the dispatcher locks the queue, pulls up to 8 tasks into a local buffer, finds the one with the smallest `duration_ms`, sends that one, and pushes the remaining tasks back to the front in their original order.

This is an approximation of Shortest Job First because it only considers a window of 8 tasks rather than the entire queue. The window size (controlled by `LOOK_AHEAD = 8` in `dispatcher.rs`) limits how aggressively long tasks are skipped, which is a deliberate fairness tradeoff.

**Trade-offs:** Under light load the queue is shallow, so the selection overhead is not justified by the savings. Under heavy load with a deep queue, the policy genuinely reduces makespan and max wait time.

---

## Arrival Over Time

Tasks do **not** all arrive at once. Each task has an `arrival_time` computed from cumulative random gaps. The main thread sleeps between each insertion based on those gaps, so tasks trickle into the queue at realistic intervals throughout the simulation.

This design makes wait time a meaningful metric. A task's wait time is the gap between when it was created and when a worker actually picked it up. The output log shows task arrivals and worker execution messages interleaved, which confirms the pipeline is running concurrently and not in a sequential batch.

---

## Experiment Design

There are two dimensions of experimentation in this project.

### Primary Experiment: FIFO vs. Optimized

The main comparison is between the two scheduling policies running on the same task set. This is the most important result in the project because it directly evaluates whether the SJF approximation improves performance over the baseline.

Both policies are tested on both workloads (balanced and stressed) to show that the benefit of the optimized policy depends on workload conditions.

### Secondary Experiment: Balanced vs. Stressed Workload

The workload variation explores how arrival rate affects performance independently of scheduling policy. Both workloads use the same random seed, so task kinds and durations are identical. Only the gap between arrivals changes.

This experiment supports the primary finding by showing that the optimized policy is more useful when the queue stays deep (stressed) than when it stays shallow (balanced).

---

## Metrics Collected

### Required Metrics

| Metric | Description |
|--------|-------------|
| Total tasks completed | Tasks that ran to completion across all workers |
| Makespan | Wall-clock time from first task creation to last completion |
| Average wait time | Mean time a task spent in the queue before execution started |
| Average turnaround time | Mean time from task arrival to full completion (wait + execution) |

### Additional Metrics

| Metric | Description |
|--------|-------------|
| Max wait time | Worst-case wait experienced by any single task |
| CPU tasks completed | Count of `Cpu`-kind tasks that finished |
| IO tasks completed | Count of `Io`-kind tasks that finished |

**Wait time** reflects queue backlog. **Turnaround time** is the total cost a task incurs from arrival to completion.

---

## Sample Results

Results are saved automatically to `.txt` files named after the policy and workload used:

- `fifo_balanced_output.txt`
- `fifo_stressed_output.txt`
- `optimized_balanced_output.txt`
- `optimized_stressed_output.txt`

### Policy Comparison: Balanced Workload (arrival gap 5 to 30 ms)

| Metric | FIFO | Optimized SJF |
|--------|------|---------------|
| Makespan | 15,734 ms | 15,844 ms |
| Avg Wait Time | 7,839.13 ms | 7,857.86 ms |
| Avg Turnaround | 7,962.65 ms | 7,981.69 ms |
| Max Wait Time | 15,555 ms | 15,663 ms |
| CPU Tasks | 259 | 259 |
| IO Tasks | 241 | 241 |

**Takeaway:** Under the balanced workload, FIFO was slightly faster. The queue stays shallow enough that the overhead of the SJF scan was not justified by the selection benefit.

### Policy Comparison: Stressed Workload (arrival gap 1 to 5 ms)

| Metric | FIFO | Optimized SJF |
|--------|------|---------------|
| Makespan | 16,153 ms | 15,905 ms |
| Avg Wait Time | 7,898.05 ms | 7,896.82 ms |
| Avg Turnaround | 8,022.78 ms | 8,021.79 ms |
| Max Wait Time | 16,081 ms | 15,771 ms |
| CPU Tasks | 247 | 247 |
| IO Tasks | 253 | 253 |

**Takeaway:** Under the stressed workload, the optimized policy reduced makespan by ~248 ms and max wait time by ~310 ms. With a consistently deep queue, the SJF window had real candidates to evaluate and the selection benefit outweighed the overhead.

### What These Results Mean

The most important finding is that the value of a scheduling optimization depends on the workload. FIFO is a strong baseline that wins under light load. The SJF approximation helps under heavy load where the queue stays deep. In both cases, worker capacity is the dominant performance constraint. Faster arrivals deepen the queue but do not meaningfully change per-task wait times because the drain rate is fixed by the workers.

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

The program prompts for two selections:

```
Select workload:
1 = Balanced workload
2 = Stressed workload
Enter choice: _

Select scheduling policy:
1 = FIFO
2 = Optimized (Shortest Job First approximation)
Enter choice: _
```

After the run completes, metrics are printed to the terminal and saved to a file named after the combination you selected (for example, `optimized_stressed_output.txt`).

---

## Configuration

The runtime prompt handles experiment selection. To adjust the underlying workload parameters, edit the relevant constants in the source:

| Parameter | File | Default (Balanced) | Default (Stressed) |
|-----------|------|--------------------|--------------------|
| Total tasks | `generator.rs` | 500 | 500 |
| Number of workers | `main.rs` | 4 | 4 |
| Arrival gap range | `generator.rs` | 5 to 30 ms | 1 to 5 ms |
| Duration range | `generator.rs` | 50 to 200 ms | 50 to 200 ms |
| SJF look-ahead window | `dispatcher.rs` | 8 | 8 |
| Random seed | `generator.rs` | 42 | 42 |

To reproduce the exact results in this README, run with the default seed (42) and the configuration above.

---

## Design Trade-offs and Limitations

### Advantages

- Modular structure: each component has exactly one job
- Safe concurrency: `Arc`, `Mutex`, and `mpsc` enforce correctness at compile time
- Two policies make direct comparison possible without changing architecture
- Output files make experiment results easy to inspect and commit

### Limitations

- FIFO causes a convoy effect where long tasks delay shorter ones
- The SJF approximation can repeatedly delay long tasks when the queue stays deep
- CPU and IO tasks share the same queue under both policies
- Fixed worker pool does not scale with load
- Round-robin assignment does not account for current worker load

---

## Future Improvements

- Separate CPU and IO queues with reserved or weighted worker pools
- Shortest-queue dispatch to route tasks to the least-busy worker instead of round-robin
- Priority scheduling with aging so long-waiting tasks gradually rise in priority
- Work stealing between workers when the queue is unbalanced
- Dynamic worker pool that scales up under high load and scales down when idle
- Per-worker utilization and idle time tracking
- Queue length statistics over time to visualize congestion patterns
- Deadline-aware scheduling for latency-sensitive task types

---

## Tool Use Disclosure

During this project, I used **ChatGPT** as development aids for architecture guidance, debugging help, module organization, and explanation support during report writing.

**Example of advice I accepted:**
Both tools recommended using `Arc<Mutex<VecDeque<Task>>>` for the shared ready queue and per-worker `mpsc` channels from the dispatcher to workers rather than having workers pull from the queue directly. This was good advice. It eliminated lock contention among workers and made the shutdown mechanism clean and implicit through channel closure.

**Example of advice I rejected or had to fix:**
An AI-generated code suggestion at one point redefined the `Task` struct directly inside `main.rs` using field names like `task.name` that did not exist in the actual `task.rs` definition. This broke the modular design and introduced type mismatch errors across multiple files. I discarded that version, kept the struct consolidated in `task.rs`, and corrected all import paths to use `use crate::task::Task`. I understand all parts of the design and can explain, modify, or debug the implementation independently.

---

## Author

**Randy Cantu**
University of Texas Rio Grande Valley
Department of Computer Science

---

## License and Academic Use

This project was created for academic coursework.
