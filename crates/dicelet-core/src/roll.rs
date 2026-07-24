use crate::constants::{MAX_DICE_FACE, MAX_DICE_NUM};
use crate::error::{DiceletError, Result};
use crate::parser::ast::KeepMode;
use crate::rng::Rng;

/// The result of rolling a single dice group (e.g. one `4d6k3`).
#[derive(Debug, Clone)]
pub struct RollResult {
    /// Individual die results
    pub results: Vec<i32>,
    /// Flags indicating which dice are kept (true) or discarded (false)
    pub flags: Vec<bool>,
    /// Sum of kept dice
    pub summary: i64,
}

impl RollResult {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            flags: Vec::new(),
            summary: 0,
        }
    }

    /// Format the detailed output for this roll, e.g. `[5 + 3 + 1* + 6]`.
    /// Discarded dice are marked with `*`.
    pub fn detail(&self) -> String {
        let parts: Vec<String> = self
            .results
            .iter()
            .zip(self.flags.iter())
            .map(|(val, kept)| {
                if *kept {
                    val.to_string()
                } else {
                    format!("{}*", val)
                }
            })
            .collect();
        format!("[{}]", parts.join(" + "))
    }
}

impl Default for RollResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Roll `num` dice with `face` sides each.
pub fn roll_base(rng: &mut dyn Rng, num: i32, face: i32) -> Result<RollResult> {
    if num < 1 {
        return Err(DiceletError::InvalidDice);
    }
    if face < 2 {
        return Err(DiceletError::InvalidDice);
    }
    if num as i64 > MAX_DICE_NUM {
        return Err(DiceletError::DiceCountExceed(num as i64, MAX_DICE_NUM));
    }
    if face as i64 > MAX_DICE_FACE {
        return Err(DiceletError::DiceFaceExceed(face as i64, MAX_DICE_FACE));
    }

    let mut result = RollResult::new();
    let mut sum: i64 = 0;

    for _ in 0..num {
        let val = rng.rand_range(1, face);
        result.results.push(val);
        result.flags.push(true);
        sum += val as i64;
    }
    result.summary = sum;

    Ok(result)
}

/// Roll `num` dice with `face` sides, keeping the highest or lowest `keep` dice.
/// `keep > 0` means keep high, `keep < 0` means keep low (the sign encodes the mode).
pub fn roll_rdk(rng: &mut dyn Rng, num: i32, face: i32, keep: i64) -> Result<RollResult> {
    let mut result = roll_base(rng, num, face)?;

    let keep_count = keep.unsigned_abs() as usize;
    if keep_count == 0 || keep_count >= result.results.len() {
        return Ok(result);
    }

    // Create indices sorted by value
    let mut indices: Vec<usize> = (0..result.results.len()).collect();

    if keep > 0 {
        // Keep highest: sort indices by value descending, mark the top N as kept
        indices.sort_by(|&a, &b| result.results[b].cmp(&result.results[a]));
    } else {
        // Keep lowest: sort indices by value ascending
        indices.sort_by(|&a, &b| result.results[a].cmp(&result.results[b]));
    }

    // Mark all as discarded first
    for flag in &mut result.flags {
        *flag = false;
    }

    // Mark the top N as kept
    let mut sum: i64 = 0;
    for &i in indices.iter().take(keep_count) {
        result.flags[i] = true;
        sum += result.results[i] as i64;
    }
    result.summary = sum;

    // Reorder results and flags to match original order (not sorted order)
    // The original implementation keeps results in roll order, just marks flags.
    // We need to restore original order — indices sorted, but results stayed in place.
    // Actually we never moved results, only sorted indices. Results are still in order.

    Ok(result)
}

/// Roll dice with keep mode.
pub fn roll_with_keep(
    rng: &mut dyn Rng,
    num: i32,
    face: i32,
    keep: Option<KeepMode>,
) -> Result<RollResult> {
    match keep {
        None => roll_base(rng, num, face),
        Some(KeepMode::High(n)) => roll_rdk(rng, num, face, n),
        Some(KeepMode::Low(n)) => roll_rdk(rng, num, face, -n),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Xoroshiro128StarStar;

    #[test]
    fn test_roll_base() {
        let mut rng = Xoroshiro128StarStar::from_seed(42);
        let result = roll_base(&mut rng, 1, 6).unwrap();
        assert_eq!(result.results.len(), 1);
        assert!(result.results[0] >= 1 && result.results[0] <= 6);
        assert!(result.flags[0]);
    }

    #[test]
    fn test_roll_rdk_high() {
        let mut rng = Xoroshiro128StarStar::from_seed(42);
        let result = roll_rdk(&mut rng, 4, 6, 3).unwrap();
        assert_eq!(result.results.len(), 4);
        assert_eq!(result.flags.iter().filter(|&&f| f).count(), 3);
    }

    #[test]
    fn test_roll_rdk_low() {
        let mut rng = Xoroshiro128StarStar::from_seed(42);
        let result = roll_rdk(&mut rng, 4, 6, -3).unwrap();
        assert_eq!(result.results.len(), 4);
        assert_eq!(result.flags.iter().filter(|&&f| f).count(), 3);
    }

    #[test]
    fn test_detail_format() {
        let mut result = RollResult::new();
        result.results = vec![5, 3, 1, 6];
        result.flags = vec![true, true, false, true];
        assert_eq!(result.detail(), "[5 + 3 + 1* + 6]");
    }

    #[test]
    fn test_limits() {
        let mut rng = Xoroshiro128StarStar::from_seed(42);
        assert!(roll_base(&mut rng, 0, 6).is_err());
        assert!(roll_base(&mut rng, 1, 1).is_err());
    }
}