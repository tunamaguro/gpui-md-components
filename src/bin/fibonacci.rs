use std::hint::black_box;

// Intentionally recursive to give the profiler a visible CPU-bound workload.
#[inline(never)]
fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(black_box(n - 1)) + fibonacci(black_box(n - 2)),
    }
}

fn main() {
    let n = std::env::args()
        .nth(1)
        .map(|value| {
            value
                .parse::<u32>()
                .expect("expected an integer from 0 to 45")
        })
        .unwrap_or(40);
    assert!(n <= 45, "expected an integer from 0 to 45");
    println!("fibonacci({n}) = {}", fibonacci(black_box(n)));
}

#[cfg(test)]
mod tests {
    use super::fibonacci;

    #[test]
    fn known_values() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(10), 55);
        assert_eq!(fibonacci(20), 6765);
    }
}
