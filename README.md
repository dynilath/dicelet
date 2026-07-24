# dice-rs

Dicelet 骰子表达式解析与求值引擎，使用 Rust 实现并通过 napi-rs 提供 Node.js / npm 接口，同时支持 WASM 用于浏览器环境。

本项目是 [qq-dicebot](https://github.com/dynilath/qq-dicebot) 中 dicelet 语法引擎的重新实现，聚焦于骰子表达式的解析与求值，支持 strtol 式容错解析。

## 特性

- 🎲 **完整的 dicelet 语法**：基础骰子、取高/取低（k/kl）、算式运算、百分数
- 📦 **多结果集合**：`#` 重复、`{}` 集合、集合间运算
- 🔧 **容错解析**：类似 `strtol` 的回退机制，从无效输入中提取有效部分
- ⚡ **高性能**：Rust 实现，xoroshiro128\*\* 随机数生成器
- 📦 **多平台支持**：
  - **Node.js**：通过 napi-rs 提供原生绑定，预编译多平台分发
  - **浏览器**：通过 WASM 支持，可在浏览器中直接使用

## 安装

### Node.js

```bash
npm install @dynilath/dicelet
```

### 浏览器 (WASM)

```bash
# 从源码构建 WASM 包
cd crates/dicelet-wasm
wasm-pack build --target web --release --out-dir pkg
```

## 快速开始

### Node.js

```typescript
import { roll, parse } from '@dynilath/dicelet';

// 基础骰子
const result = roll('4d6k3');
console.log(result.full);
// 输出: [5 + 3 + 1* + 6] = 14

// 复杂算式
const complex = roll('(((4d6+3)/2+2d20)+4*1d6)*150%');
console.log(complex.full);

// 多结果集合
const set = roll('6#4d6k3');
console.log(set.full);
console.log(set.isSet); // true

// 容错解析
const recovered = roll('d20 + (d4+ 测试');
console.log(recovered.consumed); // "d20"
console.log(recovered.tail);     // "+ (d4+ 测试"

// 仅解析不投掷
const parsed = parse('d20 + (d4+ 测试');
console.log(parsed.success);  // true

// 关闭详细输出
const noDetail = roll('4d6', { showDetail: false });
console.log(noDetail.full);   // "14"
```

### 浏览器 (WASM)

```html
<script type="module">
  import init, { roll, parse } from './pkg/dicelet_wasm.js';

  // 初始化 WASM 模块
  await init();

  // 基础骰子
  const result = roll('4d6k3');
  console.log(result.full); // e.g. "[5 + 3 + 1* + 6] = 14"

  // 容错解析
  const recovered = roll('d20 + (d4+ 测试');
  console.log(recovered.consumed); // "d20"
  console.log(recovered.tail);     // "+ (d4+ 测试"

  // 仅解析
  const parsed = parse('d20 + (d4+ 测试');
  console.log(parsed.success); // true
</script>
```

## API

### `roll(expression: string, options?: Options): RollResult`

解析并求值一个 dicelet 表达式。

**Options:**

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `showDetail` | boolean | `true` | 是否显示详细投掷结果 |
| `seed` | number | - | 随机种子（用于测试） |

**RollResult:**

| 字段 | 类型 | 说明 |
|------|------|------|
| `consumed` | string | 成功解析的源码文本 |
| `tail` | string | 未解析的尾后内容 |
| `summary` | string | 结果摘要（如 `"14"` 或 `"{10, 11, 13}"`） |
| `detail` | string | 详细投掷过程（如 `"[5 + 3 + 1* + 6]"`） |
| `full` | string | 完整输出（如 `"[5 + 3 + 1* + 6] = 14"`） |
| `isSet` | boolean | 是否为多结果集合 |
| `values` | number[] | 数值结果数组 |

### `parse(expression: string): ParseOutput`

仅解析表达式，不进行投掷。

**ParseOutput:**

| 字段 | 类型 | 说明 |
|------|------|------|
| `success` | boolean | 是否成功解析 |
| `consumed` | string | 成功解析的源码文本 |
| `tail` | string | 未解析的尾后内容 |

## 语法文档

完整的 dicelet 语法规范请参考 [docs/dicelet-syntax.md](./docs/dicelet-syntax.md)。

## 构建

### 前置要求

- Rust 工具链（stable）
- Node.js >= 10
- npm

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/dynilath/dice-rs.git
cd dice-rs

# 构建 Node.js native 模块
cd crates/dicelet-napi
npm install
npx napi build --platform --release

# 构建 WASM 模块（需要 wasm-pack）
cd ../dicelet-wasm
wasm-pack build --target web --release --out-dir pkg
```

### 运行测试

```bash
# Rust 核心引擎测试
cargo test --package dicelet-core

# Node.js 接口测试
cd crates/dicelet-napi
node test.js
```

## 项目结构

```
dice-rs/
├── crates/
│   ├── dicelet-core/          # 核心引擎（纯 Rust，无平台依赖）
│   │   └── src/
│   │       ├── lib.rs          # 入口，导出 roll() / parse()
│   │       ├── number.rs       # Number 类型（整数/小数/百分数）
│   │       ├── rng.rs          # xoroshiro128** 随机数生成器
│   │       ├── lexer/          # 词法分析
│   │       │   ├── mod.rs      # Tokenizer
│   │       │   ├── scanner.rs  # 预扫描（括号匹配截断）
│   │       │   └── token.rs    # Token 类型定义
│   │       ├── parser/         # 语法分析
│   │       │   ├── mod.rs      # 递归下降解析器
│   │       │   └── ast.rs      # AST 节点定义
│   │       ├── eval.rs         # 求值器
│   │       ├── roll.rs         # 骰子滚动
│   │       ├── error.rs        # 错误类型
│   │       └── constants.rs    # 常量
│   ├── dicelet-napi/          # napi-rs Node.js 绑定
│   │   └── src/lib.rs
│   └── dicelet-wasm/          # WASM 浏览器绑定
│       └── src/lib.rs
├── docs/
│   └── dicelet-syntax.md      # 语法文档
├── Cargo.toml                 # Rust workspace
└── package.json               # npm 包定义
```

## License

MIT