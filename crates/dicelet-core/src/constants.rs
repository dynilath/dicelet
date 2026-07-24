/// Maximum number of dice that can be rolled at once.
pub const MAX_DICE_NUM: i64 = 50;

/// Maximum number of faces on a single die.
pub const MAX_DICE_FACE: i64 = 1000;

/// Maximum repeat count for `#` (independent roll sets).
pub const MAX_DICE_UNIT_COUNT: i64 = 10;

/// Characters that are illegal in macro identifiers.
pub const ILLEGAL_IDENTIFIER_CHARS: &str = " +-*/{},()#=<>&|:%.";