# Learn CPU profiling with a small Rust program

Profiling helps answer **where does this program spend its time?** In this tutorial, you will count prime numbers, locate an expensive function with a sampling profiler, and measure an algorithmic improvement.

The commands below target macOS with a terminal and a browser. The program has no external Rust dependencies. The runnable project is already included in this directory; steps 2–3 explain how to recreate it from scratch.

## 1. Check Rust and install the profiler

```sh
rustc --version
cargo --version
cargo install --locked samply
samply --version
```

If Rust is missing, install it using the instructions at [rustup.rs](https://rustup.rs/), then reopen your terminal. On macOS, if compilation reports a missing linker, install Apple's command-line tools with `xcode-select --install`.

[Samply](https://github.com/mstange/samply) records samples of running call stacks and opens a browser-based profile viewer. Its official documentation provides the installation and recording commands used here.

## 2. Create a project

For a new project, run these commands in a directory of your choice:

```sh
cargo new profiling-demo
cd profiling-demo
```

If using the files already provided, instead run:

```sh
cd /Users/kadyapam/projects/profiling
```

Add this to `Cargo.toml` if it is not already present:

```toml
[profile.release]
debug = true
```

This keeps debug information available in an optimized release build so the profiler can resolve source locations. See the [Cargo profile documentation](https://doc.rust-lang.org/cargo/reference/profiles.html#debug).

## 3. Add the program

Replace `src/main.rs` with the following code. Both versions are included so you can compare them without editing and rebuilding between measurements.

```rust
use std::{hint::black_box, time::Instant};

// Keep these functions visible in the profiler for this teaching example.
#[inline(never)]
fn is_prime_slow(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    for divisor in 2..n {
        if n % divisor == 0 {
            return false;
        }
    }
    true
}

#[inline(never)]
fn is_prime_fast(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    let mut divisor = 2;
    // A composite number has at least one factor <= sqrt(n).
    // Division avoids overflow from divisor * divisor.
    while divisor <= n / divisor {
        if n % divisor == 0 {
            return false;
        }
        divisor += 1;
    }
    true
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("slow");
    let check: fn(u64) -> bool = match mode {
        "slow" => is_prime_slow,
        "fast" => is_prime_fast,
        _ => panic!("usage: profiling-demo [slow|fast] [positive repetitions]"),
    };
    let repetitions: u64 = args
        .get(2)
        .map_or(100, |s| s.parse().expect("integer required"));
    assert!(repetitions > 0, "repetitions must be positive");

    let start = Instant::now();
    let mut total = 0;
    for _ in 0..repetitions {
        for n in 2..=30_000 {
            // Discourage the optimizer from eliminating repeated work.
            if check(black_box(n)) {
                total += 1;
            }
        }
    }
    let elapsed = start.elapsed();
    println!("mode={mode}, repetitions={repetitions}, total={total}, elapsed={elapsed:.3?}");
}
```

`slow` checks divisors up to `n - 1`; prime numbers therefore require many divisions. `fast` stops at the square root: if `n = a × b`, at least one factor must be no larger than `sqrt(n)`.

`#[inline(never)]` makes the two functions easier to locate in a profile. `black_box` discourages removal of the repeated workload. Both are teaching aids; the algorithm change is what produces the speedup.

## 4. Build and measure a baseline

```sh
cargo build --release
./target/release/profiling-demo slow 100
```

The program checks numbers from 2 through 30,000, repeating that work 100 times. There are 3,245 primes in that range, so the output should contain `total=324500`. It also prints elapsed wall-clock time for the computation, excluding compilation and the final print.

Record the elapsed time. Timing tells you how long the work takes; profiling helps explain where that time goes.

## 5. Record the slow version

```sh
samply record ./target/release/profiling-demo slow 100
```

Profile the compiled executable directly. Running `samply record cargo run ...` would also capture Cargo's work.

After the program finishes, Samply opens its viewer in your default browser. Keep the terminal process running while inspecting the profile. If it does not open automatically, use the viewer URL printed in the terminal.

In the viewer:

1. Select the program's main thread and its busy time interval.
2. Open **Call Tree** and expand the application frames, or search for `is_prime_slow`.
3. Compare **Self** (samples in the function itself) with **Total** (samples in it and its descendants).
4. Open **Flame Graph**. A wider frame represents more sampled time; vertical position represents the call stack, not elapsed time.
5. Open the source view for `is_prime_slow`, when available, to inspect the divisor loop.

Expected finding: `is_prime_slow` should dominate this CPU-bound workload. Sample counts estimate time distribution; they are not function-call counts, and optimized source attribution can be approximate.

When finished inspecting, press Ctrl-C in the Samply terminal to stop its local server before recording another profile.

## 6. Measure the improved algorithm

```sh
./target/release/profiling-demo fast 100
```

Confirm that the output still contains `total=324500`. The `fast` implementation should take much less time because it performs far fewer divisions.

For a more useful comparison, run each version three times without the profiler:

```sh
for run in 1 2 3; do
  ./target/release/profiling-demo slow 100
  ./target/release/profiling-demo fast 100
done
```

Use the middle elapsed value for each mode and compute:

```text
speedup = median slow time / median fast time
```

Keep the repetition count, build, and machine the same. Close other CPU-heavy applications. Do not use profiled timings as your primary speed comparison because recording adds overhead. Matching totals is a useful sanity check, although it is not a complete correctness proof.

## 7. Profile the improved version

```sh
samply record ./target/release/profiling-demo fast 100
```

Locate `is_prime_fast` and inspect the shortened recording. It can still occupy most of the graph even though the program is much faster: percentages describe each recording's own workload.

If the fast run is too short to collect useful samples, increase repetitions:

```sh
samply record ./target/release/profiling-demo fast 5000
```

Use this longer run only to inspect the function. Its absolute duration and total sample count are not directly comparable with the 100-repetition baseline.

## 8. Apply the workflow elsewhere

Use the same cycle on a real application: choose representative input, time an optimized build, profile it, change the dominant expensive operation, check the results, and time the same workload again.

This example demonstrates CPU profiling. Memory use, allocations, and I/O bottlenecks need additional measurements appropriate to those resources.

## Troubleshooting

- **`samply` is not found:** reopen the terminal and check that `$HOME/.cargo/bin` is on `PATH`.
- **Missing function names or source:** confirm `debug = true`, rebuild with `cargo build --release`, and record that binary while its build artifacts and source are still available.
- **Very few samples:** increase repetitions until the computation lasts several seconds.
- **Unexpectedly slow results:** make sure you use `target/release`, not `target/debug`.
- **Linux permissions:** Samply also supports Linux, but recording may require perf-event access. Follow its [platform instructions](https://github.com/mstange/samply#description) if recording is denied.

