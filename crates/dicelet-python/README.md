# dicelet

Dicelet dice expression parsing and evaluation engine for Python.

Backed by a high-performance Rust implementation via PyO3.

## Installation

```bash
pip install dicelet
```

## Quick Start

```python
import dicelet

# Basic dice
result = dicelet.roll("4d6k3")
print(result.full)  # [5 + 3 + 1* + 6] = 14

# Complex expression
result = dicelet.roll("(((4d6+3)/2+2d20)+4*1d6)*150%")
print(result.full)

# Multi-result sets
result = dicelet.roll("6#4d6k3")
print(result.full)
print(result.is_set)  # True

# Fault-tolerant parsing: extracts valid parts from invalid input
result = dicelet.roll("d20 + (d4+ test")
print(result.consumed)  # "d20"
print(result.tail)      # "+ (d4+ test"

# Parse only (no rolling)
parsed = dicelet.parse("d20 + (d4+ test")
print(parsed.success)  # True

# Disable verbose output
result = dicelet.roll("4d6", show_detail=False)
print(result.full)  # "14"
```

## API

### `dicelet.roll(expression, show_detail=True, seed=None) -> RollResult`

Parse and evaluate a dicelet expression.

- `expression` (str): The dicelet expression string.
- `show_detail` (bool): Whether to show detailed roll results. Default True.
- `seed` (int | None): Optional random seed for deterministic results.

Returns a `RollResult` with fields: `consumed`, `tail`, `summary`, `detail`, `full`, `is_set`, `values`.

### `dicelet.parse(expression) -> ParseOutput`

Parse an expression without rolling.

Returns a `ParseOutput` with fields: `success`, `consumed`, `tail`.

## License

MIT
