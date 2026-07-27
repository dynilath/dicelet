# dicelet

Dicelet dice expression parsing and evaluation engine, implemented in Rust with napi-rs for Node.js/npm, WASM for browser environments, and Python bindings via PyO3.

This project is a reimplementation of the dicelet syntax engine from [qq-dicebot](https://github.com/dynilath/qq-dicebot), focused on dice expression parsing and evaluation with strtol-style fault-tolerant parsing.

## Features

- 🎲 **Full dicelet syntax**: basic dice, keep-highest/keep-lowest (k/kl), arithmetic, percentages
- 📦 **Multi-result sets**: `#` repetition, `{}` sets, inter-set operations
- 🔧 **Fault-tolerant parsing**: strtol-like recovery, extracting valid parts from invalid input
- ⚡ **High performance**: Rust implementation, xoroshiro128** RNG
- 📦 **Multi-platform support**:
  - **Node.js**: native bindings via napi-rs, prebuilt multi-platform distribution
  - **Browser**: WASM support for direct browser usage
  - **Python**: native bindings via PyO3, published on PyPI

## Installation

### Node.js

```bash
npm install dicelet
```

### Python

```bash
pip install dicelet
```

### Browser (WASM)

```bash
# Build WASM package from source
cd crates/dicelet-wasm
wasm-pack build --target web --release --out-dir pkg
```

## Quick Start

### Node.js

```typescript
import { roll, parse } from 'dicelet';

// Basic dice
const result = roll('4d6k3');
console.log(result.full);
// Output: [5 + 3 + 1* + 6] = 14

// Complex expression
const complex = roll('(((4d6+3)/2+2d20)+4*1d6)*150%');
console.log(complex.full);

// Multi-result sets
const set = roll('6#4d6k3');
console.log(set.full);
console.log(set.isSet); // true

// Fault-tolerant parsing
const recovered = roll('d20 + (d4+ test');
console.log(recovered.consumed); // "d20"
console.log(recovered.tail);     // "+ (d4+ test"

// Parse only (no rolling)
const parsed = parse('d20 + (d4+ test');
console.log(parsed.success);  // true

// Disable verbose output
const noDetail = roll('4d6', { showDetail: false });
console.log(noDetail.full);   // "14"
```

### Python

```python
import dicelet

# Basic dice
result = dicelet.roll("4d6k3")
print(result.full)
# Output: [5 + 3 + 1* + 6] = 14

# Complex expression
complex_result = dicelet.roll("(((4d6+3)/2+2d20)+4*1d6)*150%")
print(complex_result.full)

# Multi-result sets
set_result = dicelet.roll("6#4d6k3")
print(set_result.full)
print(set_result.is_set)  # True

# Fault-tolerant parsing
recovered = dicelet.roll("d20 + (d4+ test")
print(recovered.consumed)  # "d20"
print(recovered.tail)      # "+ (d4+ test"

# Parse only (no rolling)
parsed = dicelet.parse("d20 + (d4+ test")
print(parsed.success)  # True

# Disable verbose output
no_detail = dicelet.roll("4d6", show_detail=False)
print(no_detail.full)  # "14"
```

### Browser (WASM)

```html
<script type="module">
  import init, { roll, parse } from './pkg/dicelet_wasm.js';

  // Initialize WASM module
  await init();

  // Basic dice
  const result = roll('4d6k3');
  console.log(result.full); // e.g. "[5 + 3 + 1* + 6] = 14"

  // Fault-tolerant parsing
  const recovered = roll('d20 + (d4+ test');
  console.log(recovered.consumed); // "d20"
  console.log(recovered.tail);     // "+ (d4+ test"

  // Parse only
  const parsed = parse('d20 + (d4+ test');
  console.log(parsed.success); // true
</script>
```

## API

### `roll(expression: string, options?: Options): RollResult` (TS) / `dicelet.roll(expression, **options)` (Python)

Parse and evaluate a dicelet expression.

**Options:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `showDetail` / `show_detail` | boolean | `true` | Whether to show detailed roll results |
| `seed` | number | - | Random seed (for testing) |

**RollResult:**

| Field | Type | Description |
|-------|------|-------------|
| `consumed` | string | Successfully parsed source text |
| `tail` | string | Unparsed trailing content |
| `summary` | string | Result summary (e.g. `"14"` or `"{10, 11, 13}"`) |
| `detail` | string | Detailed roll process (e.g. `"[5 + 3 + 1* + 6]"`) |
| `full` | string | Full output (e.g. `"[5 + 3 + 1* + 6] = 14"`) |
| `isSet` / `is_set` | boolean | Whether this is a multi-result set |
| `values` | number[] | Array of numeric results |

### `parse(expression: string): ParseOutput` (TS) / `dicelet.parse(expression)` (Python)

Parse an expression without rolling.

**ParseOutput:**

| Field | Type | Description |
|-------|------|-------------|
| `success` | boolean | Whether parsing succeeded |
| `consumed` | string | Successfully parsed source text |
| `tail` | string | Unparsed trailing content |

## Syntax Documentation

See [docs/dicelet-syntax.md](./docs/dicelet-syntax.md) for the complete dicelet syntax specification.

## Building

### Prerequisites

- Rust toolchain (stable)
- Node.js >= 10
- Python >= 3.8
- npm

### Build from source

```bash
# Clone the repository
git clone https://github.com/dynilath/dicelet.git
cd dicelet

# Build Node.js native module
cd crates/dicelet-napi
npm install
npx napi build --platform --release

# Build WASM module (requires wasm-pack)
cd ../dicelet-wasm
wasm-pack build --target web --release --out-dir pkg

# Build Python module (requires maturin)
cd ../dicelet-python
pip install maturin
maturin develop --release
```

### Run tests

```bash
# Rust core engine tests
cargo test --package dicelet-core

# Node.js bindings tests
cd crates/dicelet-napi
node test.js

# Python bindings tests
cd crates/dicelet-python
python -m pytest
```

## Project Structure

```
dicelet/
├── crates/
│   ├── dicelet-core/          # Core engine (pure Rust, no platform deps)
│   │   └── src/
│   │       ├── lib.rs          # Entry point, exports roll() / parse()
│   │       ├── number.rs       # Number type (int/decimal/percent)
│   │       ├── rng.rs          # xoroshiro128** RNG
│   │       ├── lexer/          # Lexical analysis
│   │       │   ├── mod.rs      # Tokenizer
│   │       │   ├── scanner.rs  # Pre-scan (parenthesis matching cutoff)
│   │       │   └── token.rs    # Token type definitions
│   │       ├── parser/         # Syntax analysis
│   │       │   ├── mod.rs      # Recursive descent parser
│   │       │   └── ast.rs      # AST node definitions
│   │       ├── eval.rs         # Evaluator
│   │       ├── roll.rs         # Dice rolling
│   │       ├── error.rs        # Error types
│   │       └── constants.rs    # Constants
│   ├── dicelet-napi/          # napi-rs Node.js bindings
│   │   └── src/lib.rs
│   ├── dicelet-wasm/          # WASM browser bindings
│   │   └── src/lib.rs
│   └── dicelet-python/        # PyO3 Python bindings
│       ├── pyproject.toml
│       └── src/lib.rs
├── docs/
│   └── dicelet-syntax.md      # Syntax documentation
├── Cargo.toml                 # Rust workspace
├── README.md                  # English README
├── README_zh.md               # Chinese README
└── package.json               # npm package definition
```

## License

MIT
