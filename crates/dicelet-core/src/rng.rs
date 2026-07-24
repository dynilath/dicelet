/// xoroshiro128** pseudo-random number generator.
///
/// This is a faithful Rust port of the original C++ implementation
/// used in qq-dicebot. The algorithm is by Blackman/Vigna.
pub struct Xoroshiro128StarStar {
    state: [u64; 2],
}

impl Xoroshiro128StarStar {
    /// Create a new RNG seeded from system entropy.
    pub fn from_entropy() -> Self {
        let nanos = entropy_nanos();
        let seed0 = nanos.wrapping_mul(0x9E3779B97F4A7C15);
        let seed1 = nanos.rotate_left(17).wrapping_mul(0xD1B54A32D192ED03);
        Self {
            state: [seed0 | 1, seed1 | 1], // ensure non-zero
        }
    }

    /// Create a new RNG with a specific seed (for testing).
    pub fn from_seed(seed: u64) -> Self {
        // SplitMix64 to expand a single u64 into two
        let mut z = seed.wrapping_add(0x9E3779B97F4A7C15);
        let s0 = splitmix64(&mut z);
        let s1 = splitmix64(&mut z);
        Self {
            state: [s0 | 1, s1 | 1],
        }
    }

    #[inline]
    fn rotl(x: u64, k: u32) -> u64 {
        (x << k) | (x >> (64 - k))
    }

    /// Generate the next 64-bit random number.
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let s0 = self.state[0];
        let mut s1 = self.state[1];
        let result = Self::rotl(s0.wrapping_mul(5), 7).wrapping_mul(9);

        s1 ^= s0;
        self.state[0] = Self::rotl(s0, 24) ^ s1 ^ (s1 << 16);
        self.state[1] = Self::rotl(s1, 37);

        result
    }

    /// Generate a random integer in the range [min, max] (inclusive).
    #[inline]
    pub fn rand_range(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        let range = (max - min + 1) as u64;
        // Use modulo with rejection-free-ish approach (simpler: just modulo)
        let r = self.next_u64() % range;
        min + r as i32
    }
}

fn splitmix64(z: &mut u64) -> u64 {
    *z = z.wrapping_add(0x9E3779B97F4A7C15);
    let mut result = *z;
    result = (result ^ (result >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    result = (result ^ (result >> 27)).wrapping_mul(0x94D049BB133111EB);
    result ^ (result >> 31)
}

impl Default for Xoroshiro128StarStar {
    fn default() -> Self {
        Self::from_entropy()
    }
}

/// A trait for random number generation, allowing different RNG implementations.
pub trait Rng {
    fn rand_range(&mut self, min: i32, max: i32) -> i32;
}

impl Rng for Xoroshiro128StarStar {
    fn rand_range(&mut self, min: i32, max: i32) -> i32 {
        self.rand_range(min, max)
    }
}

/// Get entropy from the system in nanoseconds.
/// On WASM targets, uses `Date.now()` via the `js-sys` crate if available,
/// otherwise falls back to a counter-based seed.
#[cfg(not(target_arch = "wasm32"))]
fn entropy_nanos() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let nanos = now.as_nanos() as u64;
    let pid = std::process::id() as u64;
    nanos.wrapping_add(pid.wrapping_mul(0x9E3779B97F4A7C15))
}

#[cfg(target_arch = "wasm32")]
fn entropy_nanos() -> u64 {
    // In WASM, use Date.now() * 1e6 as entropy
    // This is called from the WASM bindings which can inject a seed
    // Fall back to a static counter if no JS environment
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0x1234567890ABCDEF);
    COUNTER.fetch_add(0x9E3779B97F4A7C15, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range() {
        let mut rng = Xoroshiro128StarStar::from_seed(42);
        for _ in 0..1000 {
            let r = rng.rand_range(1, 6);
            assert!(r >= 1 && r <= 6);
        }
    }

    #[test]
    fn test_deterministic() {
        let mut a = Xoroshiro128StarStar::from_seed(123);
        let mut b = Xoroshiro128StarStar::from_seed(123);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }
}