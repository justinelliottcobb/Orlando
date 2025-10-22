# Orlando Benchmarks

Comprehensive performance benchmarking suite comparing Orlando transducers against popular JavaScript libraries.

## Libraries Tested

- **Orlando** - WASM-powered transducers (this library)
- **Native Array** - JavaScript built-in Array.prototype methods
- **Underscore.js** - Classic utility library
- **Ramda** - Functional programming library
- **Lodash** - Modern utility library
- **Lazy.js** - Lazy evaluation library

## Quick Start

### Node.js Benchmarks

```bash
# Install dependencies
npm install

# Build Orlando for Node.js
npm run build:nodejs

# Run full benchmark suite
npm run bench:all

# Run quick benchmarks (fewer iterations)
npm run bench:quick
```

### Browser Benchmarks

```bash
# Build Orlando for web
npm run build:release

# Serve the examples directory
npx http-server examples -p 8080

# Open in browser
open http://localhost:8080/benchmark-comparison.html
```

## Benchmark Scenarios

### 1. Map → Filter → Take (100K items)

**Operation:** Double numbers, filter divisible by 3, take first 10

**Why this matters:**
- Tests early termination benefits
- Shows advantage of single-pass execution
- Most libraries must process all 100K items even though we only need 10

**Expected winner:** Orlando or Lazy.js (both support early termination)

---

### 2. Complex Pipeline (50K items, 10 operations)

**Operations:** Chain of 10 map/filter operations

**Why this matters:**
- Tests composition efficiency
- Shows overhead of intermediate arrays
- Real-world pipelines often have many operations

**Expected winner:** Orlando (single pass, zero intermediate arrays)

---

### 3. Early Termination (1M items, find first 5)

**Operation:** Filter by condition, take first 5 matches

**Why this matters:**
- **Massive** performance difference scenario
- Orlando stops after finding 5 matches (~685 items processed)
- Native arrays process all 1M items before slicing

**Expected winner:** Orlando by 10-20x margin

---

### 4. Object Processing (500K objects)

**Operation:** Multiple filters and map on complex objects

**Why this matters:**
- Real-world data processing scenario
- Tests performance with object allocations
- Common in ETL and data transformation pipelines

**Expected winner:** Orlando or Lazy.js

---

### 5. Simple Map (1M items)

**Operation:** Just double each number

**Why this matters:**
- Baseline comparison
- No early termination advantage
- Tests raw throughput

**Expected winner:** Native Array (least overhead for simple operations)

## Understanding the Results

### When Orlando Wins

Orlando shows **significant** advantages when:

1. **Early termination is possible** (`take`, `takeWhile`)
   - Orlando stops processing immediately
   - Native arrays must complete all operations first
   - Can be 10-20x faster!

2. **Complex pipelines** (3+ operations)
   - No intermediate array allocations
   - Single pass over data
   - Typically 2-5x faster

3. **Large datasets** (>10K items)
   - More data = bigger wins
   - Memory savings become significant

### When Native Arrays Win

Native arrays can be faster for:

1. **Single operations** on small datasets
   - Less overhead
   - JIT optimizations
   - No WASM initialization cost

2. **Simple transformations** (<100 items)
   - Orlando's setup overhead not worth it
   - Use arrays for small data!

### When to Use Each Library

| Library | Best For | Why |
|---------|----------|-----|
| **Orlando** | Large datasets, complex pipelines, early termination | WASM speed, single-pass execution |
| **Native Array** | Simple operations, small datasets, prototyping | Zero dependencies, familiar API |
| **Lazy.js** | Infinite sequences, early termination | Lazy evaluation, functional style |
| **Ramda** | Functional programming, immutability | Pure functions, composition |
| **Lodash** | General utilities, object manipulation | Comprehensive API, battle-tested |
| **Underscore** | Legacy projects, minimal size | Small footprint, simple API |

## Interpreting the Output

### Node.js Output

```
╔═══════════════════════════════════════════════════════════════════╗
║     Orlando Transducers - Comprehensive Benchmark Suite         ║
╚═══════════════════════════════════════════════════════════════════╝

  Map → Filter → Take (100K items)
  Dataset size: 100,000 items

┌────────────────────┬───────────────┬───────────────┬──────────────────┬──────────────────┐
│ Library            │ Ops/sec       │ Avg Time      │ vs Native        │ vs Orlando       │
├────────────────────┼───────────────┼───────────────┼──────────────────┼──────────────────┤
│ Orlando 🏆         │ 15,234        │ 0.66ms        │ 4.2x faster      │ -                │
│ Lazy.js            │ 12,891        │ 0.78ms        │ 3.5x faster      │ 1.2x slower      │
│ Native Array       │ 3,621         │ 2.76ms        │ -                │ 4.2x slower      │
│ Lodash             │ 3,512         │ 2.85ms        │ 1.0x slower      │ 4.3x slower      │
│ Underscore         │ 3,498         │ 2.86ms        │ 1.0x slower      │ 4.4x slower      │
│ Ramda              │ 2,891         │ 3.46ms        │ 1.3x slower      │ 5.2x slower      │
└────────────────────┴───────────────┴───────────────┴──────────────────┴──────────────────┘

  Summary:
  ✅ Orlando is the fastest!
  ✅ 18.2% faster than Lazy.js
```

**Key metrics:**
- **Ops/sec**: Higher is better (operations per second)
- **Avg Time**: Lower is better (average execution time)
- **vs Native**: Comparison against native arrays
- **vs Orlando**: Comparison against Orlando

### Browser Output

Visual bar charts showing relative performance. Fastest library has the longest bar and is highlighted in green.

## Running Specific Benchmarks

Edit `benchmarks/comparison.js` to comment out scenarios you don't want to run:

```javascript
const scenarios = [
    // scenarioMapFilterTake,
    scenarioComplexPipeline,
    // scenarioEarlyTermination,
    // ...
];
```

## Performance Tips

Based on benchmark results:

1. **Use Orlando for:**
   - Datasets > 1000 items
   - Pipelines with 3+ operations
   - Any scenario with early termination
   - Performance-critical code

2. **Use Native Arrays for:**
   - Datasets < 100 items
   - Single operations
   - Quick prototypes
   - Code clarity over performance

3. **Use Lazy.js for:**
   - Infinite sequences
   - Early termination with functional style
   - Alternative to Orlando without WASM

## CI/CD Integration

Benchmarks can be run in CI to track performance over time:

```yaml
# .github/workflows/benchmark.yml
name: Benchmarks

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - run: npm install
      - run: npm run build:nodejs
      - run: npm run bench:quick
```

## Contributing

To add new benchmarks:

1. Add scenario to `scenarios` array in `comparison.js`
2. Include all libraries for fair comparison
3. Use realistic data and operations
4. Document what the benchmark tests
5. Run benchmarks on multiple machines

## Troubleshooting

### "Cannot find module '../pkg/orlando.js'"

Build Orlando for Node.js first:
```bash
npm run build:nodejs
```

### Benchmarks are too slow

Use quick mode for faster iteration:
```bash
npm run bench:quick
```

### Results vary widely between runs

- Close other applications
- Run benchmarks multiple times
- Use more iterations (edit `ITERATIONS` in comparison.js)
- Disable CPU throttling

### Memory issues with large datasets

Reduce dataset sizes in the scenarios or increase Node.js memory:
```bash
node --max-old-space-size=4096 benchmarks/comparison.js
```

## Sample Results

See [BENCHMARK_RESULTS.md](./BENCHMARK_RESULTS.md) for sample output from various environments.
