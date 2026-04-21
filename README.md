# Concurrent Task Dispatcher (Rust)

## Overview

This project implements a concurrent task dispatcher in Rust that simulates how systems handle workloads over time. Tasks arrive gradually, are placed into a shared ready queue, and are distributed to worker threads using a dispatcher.

The system models real-world scheduling behavior and allows comparison between different workload conditions.

## Features

- Multi-threaded worker pool
- Shared ready queue (`Arc<Mutex<VecDeque>>`)
- Dispatcher thread for scheduling
- Task arrival over time (not batch processing)
- FIFO scheduling policy
- Round-robin task distribution across workers
- CPU-bound and IO-bound task simulation
- Runtime experiment selection (balanced vs stressed)
- Performance metrics:
  - Total tasks completed
  - Makespan
  - Average wait time
  - Average turnaround time
  - Maximum wait time
  - CPU vs IO task breakdown
- Clean thread shutdown

## System Architecture

```
Generator (main thread)
        ↓
   Ready Queue (Shared)
        ↓
   Dispatcher Thread
        ↓
 Worker Channels (per worker)
        ↓
   Worker Threads
```

## Scheduling Policy

The system uses a combination of:

- **FIFO (First-In, First-Out)**  
  Tasks are removed from the queue in order of arrival.

- **Round-Robin Dispatching**  
  Tasks are assigned evenly across worker threads using channels.

This ensures fairness and balanced distribution of work.

## Arrival Over Time

Tasks are not inserted into the queue all at once.

Instead:
- Each task has an `arrival_time`
- The main thread sleeps between insertions
- Tasks continue arriving while workers are already processing

This creates a realistic simulation of continuous workload arrival.

## Experiments

### Experiment A — Balanced Workload

- Moderate arrival rate  
- Approximately equal CPU and IO tasks  

**Expected behavior:**
- Steady processing  
- Noticeable queue buildup  
- Moderate-to-high wait times  

### Experiment B — Stressed Workload

- Faster task arrival rate  
- Same number of workers (limited capacity)  

**Expected behavior:**
- Heavy queue congestion  
- Higher wait times  
- Tasks may wait nearly the full duration of the simulation  

## Example Output

```
Total tasks completed: 500
Makespan: 27699 ms
Average wait time: 13149.97 ms
Average turnaround time: 13362.96 ms
Max wait time: 27531 ms
CPU tasks completed: 231
IO tasks completed: 269
```

## How to Run

### 1. Clone the repository

```
git clone https://github.com/yourusername/concurrent-task-dispatcher.git
cd concurrent-task-dispatcher
```

### 2. Build the project

```
cargo build
```

### 3. Run the program

```
cargo run
```

## Select Experiment

When the program runs, you will be prompted:

```
Select experiment:
1 = Balanced workload
2 = Stressed workload
```

Enter:
- `1` for balanced  
- `2` for stressed  

## Configuration

### Number of tasks

```rust
let tasks = generate_tasks(500, stressed);
```

### Number of workers

```rust
let num_workers = 4;
```

### Arrival rate (in `generator.rs`)

```rust
// Balanced workload
rng.gen_range(5..31);

// Stressed workload
rng.gen_range(1..6);
```

## Technologies Used

- Rust  
- Standard Library:
  - `std::thread`
  - `std::sync::{Arc, Mutex, mpsc}`
  - `std::collections::VecDeque`
- `rand` crate  

## Design Trade-offs

### Advantages
- Simple and predictable scheduling  
- Safe concurrency using Rust ownership model  
- Easy to extend for additional policies  

### Limitations
- FIFO does not prioritize short tasks  
- High wait times under heavy load  
- Fixed number of worker threads  

## Future Improvements

- Priority scheduling (e.g., shortest-job-first)  
- Separate CPU and IO queues  
- Dynamic worker scaling  
- Smarter load balancing  
- Real-time visualization  

## Tool Use Disclosure

This project was developed with assistance from AI tools (ChatGPT) for:

- Designing concurrency structure  
- Debugging Rust compilation issues  
- Understanding synchronization patterns  

### Accepted Example
Used `Arc<Mutex<VecDeque>>` for safe shared queue access.

### Rejected Example
An AI-generated solution redefined the `Task` struct in `main.rs`, which broke modular design and was not used.

## Author

Randy Cantu  
University of Texas Rio Grande Valley  
Computer Science  

## License

This project is for academic use.