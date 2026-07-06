/// Computes the factorial of a non-negative integer.
///
/// # Definition
/// - `0! = 1` (by convention)
/// - `n! = n * (n-1) * (n-2) * ... * 1` for `n > 0`
///
/// # Arguments
/// * `n` - A non-negative integer (u64)
///
/// # Returns
/// The factorial value as `u64`.
///
/// # Panics
/// Panics if the result overflows `u64` (i.e., `n > 20`).
///
/// # Examples
/// ```
/// assert_eq!(factorial(0), 1);
/// assert_eq!(factorial(1), 1);
/// assert_eq!(factorial(5), 120);
/// ```
pub fn factorial(n: u64) -> u64 {
    if n == 0 || n == 1 {
        return 1;
    }

    let mut result: u64 = 1;
    for i in 2..=n {
        result = result.checked_mul(i).expect("factorial overflow: n > 20");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial_zero() {
        assert_eq!(factorial(0), 1);
    }

    #[test]
    fn test_factorial_one() {
        assert_eq!(factorial(1), 1);
    }

    #[test]
    fn test_factorial_small() {
        assert_eq!(factorial(2), 2);
        assert_eq!(factorial(3), 6);
        assert_eq!(factorial(4), 24);
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(6), 720);
    }

    #[test]
    fn test_factorial_large() {
        assert_eq!(factorial(10), 3_628_800);
        assert_eq!(factorial(20), 2_432_902_008_176_640_000);
    }

    #[test]
    #[should_panic(expected = "factorial overflow")]
    fn test_factorial_overflow() {
        factorial(21);
    }
}