/**
 * Orlando v0.2.0 - Phase 2b Operations Examples
 *
 * This file demonstrates the new operations added in Phase 2b:
 * - aperture (sliding windows)
 * - takeLast (take last N elements)
 * - dropLast (drop last N elements)
 */

// Note: In a real application, you'd import from 'orlando-transducers'
// import init, { aperture, takeLast, dropLast } from 'orlando-transducers';
// await init();

console.log('=== Phase 2b Operations Examples ===\n');

// Example 1: Aperture - Sliding Windows
console.log('1. Aperture (Sliding Windows)');
console.log('------------------------------');

// Mock implementation for demonstration
function aperture(arr, size) {
    const result = [];
    for (let i = 0; i <= arr.length - size; i++) {
        result.push(arr.slice(i, i + size));
    }
    return result;
}

const temperatures = [15, 18, 22, 25, 23, 20, 17];
const windows = aperture(temperatures, 3);
console.log('Temperatures:', temperatures);
console.log('3-day windows:', windows);

// Calculate moving averages
const movingAverages = windows.map(window =>
    window.reduce((sum, temp) => sum + temp, 0) / window.length
);
console.log('Moving averages:', movingAverages.map(t => t.toFixed(1)));
console.log();

// Example 2: Aperture for N-grams
console.log('2. Aperture for N-grams');
console.log('-----------------------');

const words = 'the quick brown fox jumps'.split(' ');
const bigrams = aperture(words, 2);
console.log('Words:', words);
console.log('Bigrams:', bigrams);
console.log();

// Example 3: TakeLast - Get Recent Items
console.log('3. TakeLast - Get Recent Items');
console.log('-------------------------------');

function takeLast(arr, n) {
    return arr.slice(Math.max(0, arr.length - n));
}

const logs = [
    'Server started',
    'Request received',
    'Processing data',
    'Database query',
    'Response sent',
    'Connection closed'
];

const recentLogs = takeLast(logs, 3);
console.log('All logs:', logs.length);
console.log('Last 3 logs:', recentLogs);
console.log();

// Example 4: DropLast - Remove Trailing Elements
console.log('4. DropLast - Remove Trailing Elements');
console.log('---------------------------------------');

function dropLast(arr, n) {
    return arr.slice(0, Math.max(0, arr.length - n));
}

const measurements = [10.2, 10.5, 10.3, 10.8, 999, 999]; // Last 2 are errors
const validMeasurements = dropLast(measurements, 2);
console.log('Raw measurements:', measurements);
console.log('After dropping errors:', validMeasurements);
console.log();

// Example 5: Combining Operations - Time Series Analysis
console.log('5. Combined Example - Time Series Analysis');
console.log('------------------------------------------');

const stockPrices = [100, 102, 98, 105, 107, 103, 110, 108, 112];
const windowSize = 3;

// Calculate moving averages using aperture
const priceWindows = aperture(stockPrices, windowSize);
const movingAvgs = priceWindows.map(w =>
    w.reduce((a, b) => a + b) / w.length
);

// Get the most recent moving averages
const recentMovingAvgs = takeLast(movingAvgs, 3);

console.log('Stock prices:', stockPrices);
console.log('Moving averages (window=' + windowSize + '):',
    movingAvgs.map(p => p.toFixed(2)));
console.log('Recent 3 moving averages:',
    recentMovingAvgs.map(p => p.toFixed(2)));
console.log();

// Example 6: Real-World Use Case - Log Analysis
console.log('6. Real-World Use Case - Recent Error Detection');
console.log('-----------------------------------------------');

const systemLogs = [
    { time: '10:00', level: 'INFO', msg: 'System start' },
    { time: '10:05', level: 'INFO', msg: 'Request processed' },
    { time: '10:10', level: 'WARN', msg: 'Slow query' },
    { time: '10:15', level: 'ERROR', msg: 'Connection failed' },
    { time: '10:20', level: 'INFO', msg: 'Retry successful' },
    { time: '10:25', level: 'ERROR', msg: 'Database timeout' },
    { time: '10:30', level: 'CRITICAL', msg: 'Service down' },
];

// Get recent errors (last 5 logs)
const recentEntries = takeLast(systemLogs, 5);
const recentErrors = recentEntries.filter(log =>
    log.level === 'ERROR' || log.level === 'CRITICAL'
);

console.log('Total logs:', systemLogs.length);
console.log('Recent entries (last 5):', recentEntries.length);
console.log('Recent errors found:', recentErrors.length);
recentErrors.forEach(err =>
    console.log(`  ${err.time} [${err.level}] ${err.msg}`)
);
console.log();

// Example 7: Aperture for Comparison
console.log('7. Aperture - Comparing Adjacent Values');
console.log('----------------------------------------');

const values = [1, 3, 2, 5, 4, 8, 6];
const pairs = aperture(values, 2);

// Find increases
const increases = pairs.filter(([a, b]) => b > a);
console.log('Values:', values);
console.log('Pairs:', pairs);
console.log('Increases:', increases);
console.log();

console.log('=== Examples Complete ===');
console.log('\nThese operations are now available in Orlando v0.2.0!');
console.log('Import them from orlando-transducers and use with your data.');
