//! Deterministic RNG owned by this crate.
//!
//! A seeded worksheet URL must produce the same problems forever, so
//! both the generator and every sampling algorithm live here rather
//! than in `rand` (whose `SmallRng` and `gen_range`/`shuffle`
//! algorithms are explicitly allowed to change between releases).
//!
//! Not cryptographic, and not trying to be: SplitMix64 for the stream,
//! multiply-shift for bounded integers. Each call consumes exactly one
//! `next_u64`, so the draw count is easy to reason about when a
//! generator changes.
//!
//! **Don't change the output of anything in this file.** Every seeded
//! worksheet depends on it; the tests below pin it.

use std::ops::{Range, RangeInclusive};

pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn seed_from_u64(seed: u64) -> Self {
        Rng { state: seed }
    }

    /// Seeded when `seed` is given, otherwise from process-local
    /// randomness (std's per-instance hasher keys).
    pub fn from_seed(seed: Option<u64>) -> Self {
        Self::seed_from_u64(seed.unwrap_or_else(|| {
            use std::hash::{BuildHasher, Hasher};
            std::collections::hash_map::RandomState::new()
                .build_hasher()
                .finish()
        }))
    }

    /// SplitMix64 (Steele, Lea & Flood, 2014).
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, n)` by multiply-shift. Bias is at most n / 2^64 —
    /// irrelevant at worksheet sizes. Panics on `n == 0`.
    fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0, "empty range");
        ((self.next_u64() as u128 * n as u128) >> 64) as u64
    }

    /// Uniform over `lo..hi` or `lo..=hi`. Panics on an empty range.
    pub fn gen_range<T: SampleInt, R: IntRange<T>>(&mut self, range: R) -> T {
        let (lo, hi) = range.bounds_inclusive();
        let (lo, hi) = (lo.to_i128(), hi.to_i128());
        assert!(lo <= hi, "empty range");
        let span = (hi - lo + 1) as u128;
        let offset = if span > u64::MAX as u128 {
            self.next_u64() as u128
        } else {
            self.below(span as u64) as u128
        };
        T::from_i128(lo + offset as i128)
    }

    /// True with probability `p` (clamped to `[0, 1]`).
    pub fn gen_bool(&mut self, p: f64) -> bool {
        // Top 53 bits as a uniform f64 in [0, 1).
        let x = (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64);
        x < p
    }

    /// Fisher–Yates, back to front.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            items.swap(i, j);
        }
    }
}

/// Integer types `gen_range` can return.
pub trait SampleInt: Copy {
    fn to_i128(self) -> i128;
    fn from_i128(v: i128) -> Self;
}

macro_rules! sample_int {
    ($($t:ty),*) => {$(
        impl SampleInt for $t {
            fn to_i128(self) -> i128 { self as i128 }
            fn from_i128(v: i128) -> Self { v as $t }
        }
    )*};
}
sample_int!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

/// `a..b` and `a..=b`, normalised to inclusive bounds.
pub trait IntRange<T> {
    fn bounds_inclusive(self) -> (T, T);
}

impl<T: SampleInt> IntRange<T> for Range<T> {
    fn bounds_inclusive(self) -> (T, T) {
        let (lo, hi) = (self.start.to_i128(), self.end.to_i128());
        assert!(lo < hi, "empty range");
        (self.start, T::from_i128(hi - 1))
    }
}

impl<T: SampleInt> IntRange<T> for RangeInclusive<T> {
    fn bounds_inclusive(self) -> (T, T) {
        self.into_inner()
    }
}

/// Lets existing `slice.shuffle(&mut rng)` call sites stay as written.
pub trait SliceShuffle {
    fn shuffle(&mut self, rng: &mut Rng);
}

impl<T> SliceShuffle for [T] {
    fn shuffle(&mut self, rng: &mut Rng) {
        rng.shuffle(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pins the raw stream. If this fails, every seeded worksheet
    /// changed — fix the code, don't update the numbers.
    #[test]
    fn splitmix64_stream_is_pinned() {
        let mut r = Rng::seed_from_u64(0);
        let got: Vec<u64> = (0..3).map(|_| r.next_u64()).collect();
        // Reference values for SplitMix64 seeded with 0.
        assert_eq!(
            got,
            [0xE220_A839_7B1D_CDAF, 0x6E78_9E6A_A1B9_65F4, 0x06C4_5D18_8009_454F]
        );
    }

    #[test]
    fn sampling_is_pinned() {
        let mut r = Rng::seed_from_u64(42);
        let ints: Vec<u32> = (0..8).map(|_| r.gen_range(1..=10)).collect();
        let signed: Vec<i64> = (0..4).map(|_| r.gen_range(-5..5)).collect();
        let bools: Vec<bool> = (0..4).map(|_| r.gen_bool(0.5)).collect();
        let mut v: Vec<u32> = (0..8).collect();
        r.shuffle(&mut v);
        assert_eq!(
            format!("{ints:?} {signed:?} {bools:?} {v:?}"),
            "[8, 2, 3, 4, 1, 9, 3, 9] [-2, 1, -3, -1] [false, false, false, true] [2, 1, 5, 4, 6, 7, 3, 0]",
        );
    }

    #[test]
    fn ranges_stay_in_bounds() {
        let mut r = Rng::seed_from_u64(7);
        for _ in 0..10_000 {
            let a: u32 = r.gen_range(3..=5);
            assert!((3..=5).contains(&a));
            let b: i64 = r.gen_range(-2..2);
            assert!((-2..2).contains(&b));
            assert_eq!(r.gen_range(9u32..=9), 9);
        }
    }
}
