/**
 * Orlando Transducers - Comprehensive Benchmark Suite
 *
 * Compares Orlando against popular JavaScript libraries:
 * - Native Array methods
 * - Underscore.js
 * - Ramda
 * - Lodash
 * - Lazy.js
 */

const { Bench } = require('tinybench');
const _ = require('underscore');
const R = require('ramda');
const lodash = require('lodash');
const Lazy = require('lazy.js');
const chalk = require('chalk');
const Table = require('cli-table3');

// Import Orlando (built with wasm-pack --target nodejs)
const { Pipeline } = require('../pkg/orlando_transducers.js');

// Benchmark configuration
const QUICK_MODE = process.argv.includes('--quick');
const ITERATIONS = QUICK_MODE ? 100 : 1000;
const WARMUP_ITERATIONS = QUICK_MODE ? 10 : 100;

// Data generators
function generateNumbers(size) {
    return Array.from({ length: size }, (_, i) => i + 1);
}

function generateObjects(size) {
    return Array.from({ length: size }, (_, i) => ({
        id: i,
        value: Math.random() * 1000,
        active: Math.random() > 0.5,
        name: `Item ${i}`,
        category: ['A', 'B', 'C'][i % 3]
    }));
}

// Benchmark scenarios
const scenarios = [
    {
        name: 'Map → Filter → Take (100K items)',
        setup: () => generateNumbers(100000),
        operations: {
            'Orlando': (data) => {
                const pipeline = new Pipeline()
                    .map(x => x * 2)
                    .filter(x => x % 3 === 0)
                    .take(10);
                return pipeline.toArray(data);
            },
            'Native Array': (data) => {
                return data
                    .map(x => x * 2)
                    .filter(x => x % 3 === 0)
                    .slice(0, 10);
            },
            'Underscore': (data) => {
                return _.chain(data)
                    .map(x => x * 2)
                    .filter(x => x % 3 === 0)
                    .first(10)
                    .value();
            },
            'Ramda': (data) => {
                return R.pipe(
                    R.map(x => x * 2),
                    R.filter(x => x % 3 === 0),
                    R.take(10)
                )(data);
            },
            'Lodash': (data) => {
                return lodash.chain(data)
                    .map(x => x * 2)
                    .filter(x => x % 3 === 0)
                    .take(10)
                    .value();
            },
            'Lazy.js': (data) => {
                return Lazy(data)
                    .map(x => x * 2)
                    .filter(x => x % 3 === 0)
                    .take(10)
                    .toArray();
            }
        }
    },
    {
        name: 'Complex Pipeline (50K items, 10 operations)',
        setup: () => generateNumbers(50000),
        operations: {
            'Orlando': (data) => {
                const pipeline = new Pipeline()
                    .map(x => x + 1)
                    .filter(x => x % 2 === 0)
                    .map(x => x * 3)
                    .filter(x => x > 10)
                    .map(x => x / 2)
                    .filter(x => x % 5 === 0)
                    .map(x => x - 1)
                    .filter(x => x < 10000)
                    .map(x => x * 2)
                    .take(100);
                return pipeline.toArray(data);
            },
            'Native Array': (data) => {
                return data
                    .map(x => x + 1)
                    .filter(x => x % 2 === 0)
                    .map(x => x * 3)
                    .filter(x => x > 10)
                    .map(x => x / 2)
                    .filter(x => x % 5 === 0)
                    .map(x => x - 1)
                    .filter(x => x < 10000)
                    .map(x => x * 2)
                    .slice(0, 100);
            },
            'Underscore': (data) => {
                return _.chain(data)
                    .map(x => x + 1)
                    .filter(x => x % 2 === 0)
                    .map(x => x * 3)
                    .filter(x => x > 10)
                    .map(x => x / 2)
                    .filter(x => x % 5 === 0)
                    .map(x => x - 1)
                    .filter(x => x < 10000)
                    .map(x => x * 2)
                    .first(100)
                    .value();
            },
            'Ramda': (data) => {
                return R.pipe(
                    R.map(x => x + 1),
                    R.filter(x => x % 2 === 0),
                    R.map(x => x * 3),
                    R.filter(x => x > 10),
                    R.map(x => x / 2),
                    R.filter(x => x % 5 === 0),
                    R.map(x => x - 1),
                    R.filter(x => x < 10000),
                    R.map(x => x * 2),
                    R.take(100)
                )(data);
            },
            'Lodash': (data) => {
                return lodash.chain(data)
                    .map(x => x + 1)
                    .filter(x => x % 2 === 0)
                    .map(x => x * 3)
                    .filter(x => x > 10)
                    .map(x => x / 2)
                    .filter(x => x % 5 === 0)
                    .map(x => x - 1)
                    .filter(x => x < 10000)
                    .map(x => x * 2)
                    .take(100)
                    .value();
            },
            'Lazy.js': (data) => {
                return Lazy(data)
                    .map(x => x + 1)
                    .filter(x => x % 2 === 0)
                    .map(x => x * 3)
                    .filter(x => x > 10)
                    .map(x => x / 2)
                    .filter(x => x % 5 === 0)
                    .map(x => x - 1)
                    .filter(x => x < 10000)
                    .map(x => x * 2)
                    .take(100)
                    .toArray();
            }
        }
    },
    {
        name: 'Early Termination (1M items, find first 5)',
        setup: () => generateNumbers(1000000),
        operations: {
            'Orlando': (data) => {
                const pipeline = new Pipeline()
                    .filter(x => x % 137 === 0)
                    .take(5);
                return pipeline.toArray(data);
            },
            'Native Array': (data) => {
                return data
                    .filter(x => x % 137 === 0)
                    .slice(0, 5);
            },
            'Underscore': (data) => {
                return _.chain(data)
                    .filter(x => x % 137 === 0)
                    .first(5)
                    .value();
            },
            'Ramda': (data) => {
                return R.pipe(
                    R.filter(x => x % 137 === 0),
                    R.take(5)
                )(data);
            },
            'Lodash': (data) => {
                return lodash.chain(data)
                    .filter(x => x % 137 === 0)
                    .take(5)
                    .value();
            },
            'Lazy.js': (data) => {
                return Lazy(data)
                    .filter(x => x % 137 === 0)
                    .take(5)
                    .toArray();
            }
        }
    },
    {
        name: 'Object Processing (500K objects)',
        setup: () => generateObjects(500000),
        operations: {
            'Orlando': (data) => {
                const pipeline = new Pipeline()
                    .filter(item => item.active)
                    .filter(item => item.value > 100)
                    .map(item => ({ id: item.id, score: item.value * 2 }))
                    .filter(item => item.score < 1500)
                    .take(1000);
                return pipeline.toArray(data);
            },
            'Native Array': (data) => {
                return data
                    .filter(item => item.active)
                    .filter(item => item.value > 100)
                    .map(item => ({ id: item.id, score: item.value * 2 }))
                    .filter(item => item.score < 1500)
                    .slice(0, 1000);
            },
            'Underscore': (data) => {
                return _.chain(data)
                    .filter(item => item.active)
                    .filter(item => item.value > 100)
                    .map(item => ({ id: item.id, score: item.value * 2 }))
                    .filter(item => item.score < 1500)
                    .first(1000)
                    .value();
            },
            'Ramda': (data) => {
                return R.pipe(
                    R.filter(item => item.active),
                    R.filter(item => item.value > 100),
                    R.map(item => ({ id: item.id, score: item.value * 2 })),
                    R.filter(item => item.score < 1500),
                    R.take(1000)
                )(data);
            },
            'Lodash': (data) => {
                return lodash.chain(data)
                    .filter(item => item.active)
                    .filter(item => item.value > 100)
                    .map(item => ({ id: item.id, score: item.value * 2 }))
                    .filter(item => item.score < 1500)
                    .take(1000)
                    .value();
            },
            'Lazy.js': (data) => {
                return Lazy(data)
                    .filter(item => item.active)
                    .filter(item => item.value > 100)
                    .map(item => ({ id: item.id, score: item.value * 2 }))
                    .filter(item => item.score < 1500)
                    .take(1000)
                    .toArray();
            }
        }
    },
    {
        name: 'Simple Map (1M items)',
        setup: () => generateNumbers(1000000),
        operations: {
            'Orlando': (data) => {
                const pipeline = new Pipeline()
                    .map(x => x * 2);
                return pipeline.toArray(data);
            },
            'Native Array': (data) => {
                return data.map(x => x * 2);
            },
            'Underscore': (data) => {
                return _.map(data, x => x * 2);
            },
            'Ramda': (data) => {
                return R.map(x => x * 2, data);
            },
            'Lodash': (data) => {
                return lodash.map(data, x => x * 2);
            },
            'Lazy.js': (data) => {
                return Lazy(data).map(x => x * 2).toArray();
            }
        }
    }
];

// Format numbers with commas
function formatNumber(num) {
    return num.toLocaleString('en-US', { maximumFractionDigits: 2 });
}

// Calculate speedup
function calculateSpeedup(baseline, current) {
    const speedup = baseline / current;
    if (speedup >= 1) {
        return chalk.green(`${speedup.toFixed(2)}x faster`);
    } else {
        return chalk.red(`${(1/speedup).toFixed(2)}x slower`);
    }
}

// Run benchmarks for a scenario
async function runScenario(scenario) {
    console.log(chalk.bold.cyan(`\n${'='.repeat(80)}`));
    console.log(chalk.bold.cyan(`  ${scenario.name}`));
    console.log(chalk.bold.cyan(`${'='.repeat(80)}\n`));

    const bench = new Bench({
        iterations: ITERATIONS,
        warmupIterations: WARMUP_ITERATIONS
    });

    // Set up test data
    const data = scenario.setup();
    console.log(chalk.gray(`  Dataset size: ${formatNumber(data.length)} items\n`));

    // Add all operations to the benchmark
    for (const [name, operation] of Object.entries(scenario.operations)) {
        bench.add(name, () => {
            operation(data.slice()); // Clone to avoid mutations
        });
    }

    // Run the benchmark
    await bench.run();

    // Sort results by speed (fastest first)
    const results = bench.tasks
        .sort((a, b) => a.result.mean - b.result.mean)
        .map(task => ({
            name: task.name,
            mean: task.result.mean,
            hz: task.result.hz,
            ops: task.result.period
        }));

    // Create results table
    const table = new Table({
        head: [
            chalk.bold('Library'),
            chalk.bold('Ops/sec'),
            chalk.bold('Avg Time'),
            chalk.bold('vs Native'),
            chalk.bold('vs Orlando')
        ],
        colWidths: [20, 15, 15, 18, 18]
    });

    const nativeResult = results.find(r => r.name === 'Native Array');
    const orlandoResult = results.find(r => r.name === 'Orlando');

    for (const result of results) {
        const isWinner = result === results[0];
        const nameStyle = isWinner ? chalk.bold.green : chalk.white;

        table.push([
            nameStyle(result.name + (isWinner ? ' 🏆' : '')),
            formatNumber(result.hz),
            `${(result.mean * 1000).toFixed(2)}ms`,
            result.name === 'Native Array'
                ? chalk.gray('-')
                : calculateSpeedup(nativeResult.mean, result.mean),
            result.name === 'Orlando'
                ? chalk.gray('-')
                : calculateSpeedup(orlandoResult.mean, result.mean)
        ]);
    }

    console.log(table.toString());

    // Summary
    console.log(chalk.bold('\n  Summary:'));
    if (orlandoResult === results[0]) {
        console.log(chalk.green(`  ✅ Orlando is the fastest!`));
        if (results.length > 1) {
            const secondPlace = results[1];
            const margin = ((secondPlace.mean - orlandoResult.mean) / orlandoResult.mean * 100).toFixed(1);
            console.log(chalk.green(`  ✅ ${margin}% faster than ${secondPlace.name}`));
        }
    } else {
        const winner = results[0];
        const margin = ((orlandoResult.mean - winner.mean) / winner.mean * 100).toFixed(1);
        console.log(chalk.yellow(`  ⚠️  ${winner.name} is ${margin}% faster than Orlando`));
    }
}

// Main execution
async function main() {
    console.log(chalk.bold.blue('\n╔═══════════════════════════════════════════════════════════════════╗'));
    console.log(chalk.bold.blue('║     Orlando Transducers - Comprehensive Benchmark Suite         ║'));
    console.log(chalk.bold.blue('╚═══════════════════════════════════════════════════════════════════╝\n'));

    console.log(chalk.gray(`  Mode: ${QUICK_MODE ? 'Quick' : 'Full'}`));
    console.log(chalk.gray(`  Iterations: ${ITERATIONS}`));
    console.log(chalk.gray(`  Warmup: ${WARMUP_ITERATIONS}\n`));

    const startTime = Date.now();

    for (const scenario of scenarios) {
        await runScenario(scenario);
    }

    const elapsed = ((Date.now() - startTime) / 1000).toFixed(1);
    console.log(chalk.bold.blue(`\n${'='.repeat(80)}`));
    console.log(chalk.bold.blue(`  Benchmarks completed in ${elapsed}s`));
    console.log(chalk.bold.blue(`${'='.repeat(80)}\n`));

    console.log(chalk.bold('  Libraries tested:'));
    console.log(chalk.gray('  - Orlando (WASM transducers)'));
    console.log(chalk.gray('  - Native Array methods'));
    console.log(chalk.gray('  - Underscore.js'));
    console.log(chalk.gray('  - Ramda'));
    console.log(chalk.gray('  - Lodash'));
    console.log(chalk.gray('  - Lazy.js\n'));
}

// Run benchmarks
main().catch(err => {
    console.error(chalk.red('Error running benchmarks:'), err);
    process.exit(1);
});
