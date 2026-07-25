use crate::constants::{MAX_DICE_FACE, MAX_DICE_NUM};
use crate::error::{DiceletError, Result};
use crate::number::Number;
use crate::parser::ast::{ComparisonOp, KeepMode};
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
    /// Which dice triggered a bonus roll (true = triggered a bonus roll). 1:1 with `results`.
    pub triggers: Vec<bool>,
}

impl RollResult {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            flags: Vec::new(),
            summary: 0,
            triggers: Vec::new(),
        }
    }

    /// Format the detailed output for this roll, e.g. `[5 + 3 + 1* + 6]`.
    /// Discarded dice are marked with `*`.
    /// Dice that triggered a bonus roll are marked with `!` (e.g. `6!`).
    pub fn detail(&self) -> String {
        let parts: Vec<String> = self
            .results
            .iter()
            .zip(self.flags.iter())
            .zip(self.triggers.iter())
            .map(|((val, kept), triggered)| {
                let s = if *kept {
                    val.to_string()
                } else {
                    format!("{}*", val)
                };
                if *triggered {
                    format!("{}!", s)
                } else {
                    s
                }
            })
            .collect();
        format!("[{}]", parts.join(" + "))
    }

    /// Format the detailed output for comparison mode, e.g. `<3* + 4 + 2* + 6>`.
    /// Shows only kept dice. Among kept dice, those NOT matching the comparison are
    /// marked with `*`. Dice that triggered a bonus roll are marked with `!`.
    pub fn detail_comparison(&self, comp: &ComparisonOp) -> String {
        let parts: Vec<String> = self
            .results
            .iter()
            .zip(self.flags.iter())
            .zip(self.triggers.iter())
            .filter(|((_, kept), _)| **kept) // only show kept dice
            .map(|((val, _kept), triggered)| {
                let matches = match comp {
                    ComparisonOp::Greater(n) => (*val as i64) > n.to_i64(),
                    ComparisonOp::GreaterEqual(n) => (*val as i64) >= n.to_i64(),
                    ComparisonOp::Less(n) => (*val as i64) < n.to_i64(),
                    ComparisonOp::LessEqual(n) => (*val as i64) <= n.to_i64(),
                    ComparisonOp::NotEqual(n) => (*val as i64) != n.to_i64(),
                };
                let base = if matches {
                    val.to_string()
                } else {
                    format!("{}*", val)
                };
                if *triggered {
                    format!("{}!", base)
                } else {
                    base
                }
            })
            .collect();
        format!("<{}>", parts.join(" + "))
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
        result.triggers.push(false);
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

    Ok(result)
}

/// Roll dice with bonus threshold.
/// For each die whose result >= bonus_threshold, roll an extra die recursively.
/// Total dice count is capped at `MAX_DICE_NUM`.
fn roll_with_bonus(
    rng: &mut dyn Rng,
    num: i32,
    face: i32,
    bonus_threshold: i64,
) -> Result<RollResult> {
    if num < 1 {
        return Err(DiceletError::InvalidDice);
    }
    if face < 2 {
        return Err(DiceletError::InvalidDice);
    }

    let mut result = RollResult::new();
    let mut roll_index = 0;

    // Roll initial dice
    for _ in 0..num {
        if result.results.len() as i64 >= MAX_DICE_NUM {
            break;
        }
        let val = rng.rand_range(1, face);
        result.results.push(val);
        result.flags.push(true);
        result.triggers.push(false);
    }

    // Process bonus dice: for each die (including newly added ones),
    // if its value >= threshold, roll an extra die
    while roll_index < result.results.len() {
        if result.results.len() as i64 >= MAX_DICE_NUM {
            break;
        }
        let val = result.results[roll_index];
        if val as i64 >= bonus_threshold {
            // This die triggered a bonus
            result.triggers[roll_index] = true;
            // Roll the bonus die
            let bonus_val = rng.rand_range(1, face);
            result.results.push(bonus_val);
            result.flags.push(true);
            result.triggers.push(false);
        }
        roll_index += 1;
    }

    // Compute summary (sum of all kept dice — all dice are kept in bonus mode)
    result.summary = result.results.iter().map(|&v| v as i64).sum();

    Ok(result)
}

/// Roll dice with optional bonus and keep modifiers.
/// The order is: bonus dice are rolled first, then keep is applied.
pub fn roll_with_modifiers(
    rng: &mut dyn Rng,
    num: i32,
    face: i32,
    keep: Option<KeepMode>,
    bonus: Option<Number>,
) -> Result<RollResult> {
    let result = match (bonus, keep) {
        (Some(bonus_threshold), _) => {
            // Roll with bonus first
            let mut result = roll_with_bonus(rng, num, face, bonus_threshold.to_i64())?;
            // Then apply keep if specified
            if let Some(keep_mode) = keep {
                apply_keep_to_result(&mut result, keep_mode);
            }
            result
        }
        (None, keep_mode) => {
            // No bonus — use existing keep logic
            roll_with_keep(rng, num, face, keep_mode)?
        }
    };

    Ok(result)
}

/// Apply keep mode to an existing RollResult (modifies flags and summary in-place).
fn apply_keep_to_result(result: &mut RollResult, keep: KeepMode) {
    let keep_val = match keep {
        KeepMode::High(n) => n,
        KeepMode::Low(n) => -n,
    };

    let keep_count = keep_val.unsigned_abs() as usize;
    if keep_count == 0 || keep_count >= result.results.len() {
        return;
    }

    // Create indices sorted by value
    let mut indices: Vec<usize> = (0..result.results.len()).collect();

    if keep_val > 0 {
        // Keep highest: sort by value descending
        indices.sort_by(|&a, &b| result.results[b].cmp(&result.results[a]));
    } else {
        // Keep lowest: sort by value ascending
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
        assert!(!result.triggers[0]);
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
        result.triggers = vec![false, false, false, false];
        assert_eq!(result.detail(), "[5 + 3 + 1* + 6]");
    }

    #[test]
    fn test_detail_with_triggers() {
        let mut result = RollResult::new();
        result.results = vec![1, 6, 5, 4];
        result.flags = vec![true, true, true, true];
        result.triggers = vec![false, true, true, false];
        assert_eq!(result.detail(), "[1 + 6! + 5! + 4]");
    }

    #[test]
    fn test_detail_comparison_format() {
        let mut result = RollResult::new();
        result.results = vec![3, 4, 2, 6];
        result.flags = vec![true, true, true, true];
        result.triggers = vec![false, false, false, false];
        let comp = ComparisonOp::Greater(Number::Int(3));
        assert_eq!(result.detail_comparison(&comp), "<3* + 4 + 2* + 6>");
    }

    #[test]
    fn test_detail_comparison_with_bonus() {
        let mut result = RollResult::new();
        result.results = vec![1, 6, 5, 4];
        result.flags = vec![false, true, true, true]; // keep top 3 (6,5,4)
        result.triggers = vec![false, true, true, false];
        let comp = ComparisonOp::Greater(Number::Int(3));
        assert_eq!(result.detail_comparison(&comp), "<6! + 5! + 4>");
    }

    #[test]
    fn test_detail_comparison_some_matching() {
        let mut result = RollResult::new();
        result.results = vec![6, 3, 1];
        result.flags = vec![true, true, true]; // all kept
        result.triggers = vec![true, false, false];
        let comp = ComparisonOp::Greater(Number::Int(3));
        assert_eq!(result.detail_comparison(&comp), "<6! + 3* + 1*>");
    }

    #[test]
    fn test_limits() {
        let mut rng = Xoroshiro128StarStar::from_seed(42);
        assert!(roll_base(&mut rng, 0, 6).is_err());
        assert!(roll_base(&mut rng, 1, 1).is_err());
    }

    #[test]
    fn test_roll_with_bonus_basic() {
        let mut rng = Xoroshiro128StarStar::from_seed(42);
        let result = roll_with_bonus(&mut rng, 2, 6, 5).unwrap();
        // With seed 42, roll 2d6 with bonus threshold 5
        // Results should have at least the 2 base dice, possibly more
        assert!(result.results.len() >= 2);
        assert_eq!(result.results.len(), result.flags.len());
        assert_eq!(result.results.len(), result.triggers.len());
    }

    #[test]
    fn test_roll_with_modifiers_bonus_only() {
        let mut rng = Xoroshiro128StarStar::from_seed(42);
        let result = roll_with_modifiers(
            &mut rng, 2, 6, None, Some(Number::Int(5)),
        ).unwrap();
        assert!(result.results.len() >= 2);
        assert!(result.flags.iter().all(|&f| f)); // all kept
    }

    #[test]
    fn test_roll_with_modifiers_bonus_and_keep() {
        let mut rng = Xoroshiro128StarStar::from_seed(42);
        let result = roll_with_modifiers(
            &mut rng, 2, 6, Some(KeepMode::High(3)), Some(Number::Int(5)),
        ).unwrap();
        assert!(result.results.len() >= 2);
        let kept_count = result.flags.iter().filter(|&&f| f).count();
        assert_eq!(kept_count, 3.min(result.results.len()));
    }
}