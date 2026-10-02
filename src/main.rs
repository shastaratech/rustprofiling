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
