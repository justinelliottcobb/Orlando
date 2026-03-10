# Data Processing Pipelines

Real-world patterns for building data transformation pipelines with Orlando.

## ETL: Extract, Transform, Load

### Normalizing User Data

```javascript
import init, { Pipeline, both } from 'orlando-transducers';
await init();

const normalizeUsers = new Pipeline()
  .filter(u => u != null)
  .filter(u => u.email != null && u.email.includes('@'))
  .map(u => ({
    id: u.id,
    name: u.name.trim(),
    email: u.email.toLowerCase().trim(),
    role: u.role || 'user',
    createdAt: new Date(u.created_at).toISOString(),
  }))
  .unique();  // deduplicate consecutive entries

// Reuse on multiple data sources
const fromCsv = normalizeUsers.toArray(csvRecords);
const fromApi = normalizeUsers.toArray(apiResponse.users);
```

### Log Processing

```javascript
// Parse and filter error logs
const errorPipeline = new Pipeline()
  .map(line => {
    const [timestamp, level, ...messageParts] = line.split(' ');
    return { timestamp, level, message: messageParts.join(' ') };
  })
  .filter(entry => entry.level === 'ERROR' || entry.level === 'FATAL')
  .map(entry => ({
    ...entry,
    timestamp: new Date(entry.timestamp),
  }));

const errors = errorPipeline.toArray(logLines);
```

## Analytics Aggregation

### Revenue Calculation

```javascript
const revenuePipeline = new Pipeline()
  .filter(event => event.type === 'purchase')
  .filter(event => event.status === 'completed')
  .map(event => event.amount);

const totalRevenue = revenuePipeline.reduce(
  events,
  (sum, amount) => sum + amount,
  0
);
```

### Top Products by Category

```javascript
import { Pipeline, sortBy, topK } from 'orlando-transducers';

// Extract and score products
const scoredProducts = new Pipeline()
  .filter(p => p.inStock && p.rating >= 3.0)
  .map(p => ({
    ...p,
    score: p.rating * Math.log(p.salesCount + 1),
  }))
  .toArray(products);

// Get top 10 by computed score
const top10 = topK(scoredProducts, 10);
```

### Funnel Analysis

```javascript
// Count users at each stage of a conversion funnel
const stages = ['visit', 'signup', 'activate', 'purchase'];

const funnelCounts = stages.map(stage => {
  const count = new Pipeline()
    .filter(event => event.stage === stage)
    .unique()  // deduplicate by consecutive user
    .toArray(events)
    .length;

  return { stage, count };
});
```

## Pagination

```javascript
function paginate(data, page, pageSize) {
  return new Pipeline()
    .drop((page - 1) * pageSize)
    .take(pageSize)
    .toArray(data);
}

const page2 = paginate(users, 2, 20);  // items 21-40
```

### Filtered Pagination

```javascript
function searchAndPaginate(data, query, page, pageSize) {
  const pipeline = new Pipeline()
    .filter(item => item.name.toLowerCase().includes(query.toLowerCase()))
    .filter(item => item.active)
    .drop((page - 1) * pageSize)
    .take(pageSize);

  return pipeline.toArray(data);
}
```

## Search with Multiple Filters

```javascript
import { Pipeline, both, allPass } from 'orlando-transducers';

function searchProducts(catalog, filters) {
  let pipeline = new Pipeline();

  if (filters.category) {
    pipeline = pipeline.filter(p => p.category === filters.category);
  }

  if (filters.minPrice != null) {
    pipeline = pipeline.filter(p => p.price >= filters.minPrice);
  }

  if (filters.maxPrice != null) {
    pipeline = pipeline.filter(p => p.price <= filters.maxPrice);
  }

  if (filters.minRating) {
    pipeline = pipeline.filter(p => p.rating >= filters.minRating);
  }

  if (filters.inStockOnly) {
    pipeline = pipeline.filter(p => p.inStock);
  }

  return pipeline.take(filters.limit || 20).toArray(catalog);
}

const results = searchProducts(catalog, {
  category: 'electronics',
  minPrice: 50,
  maxPrice: 500,
  minRating: 4.0,
  inStockOnly: true,
  limit: 20,
});
```

## Combining Multiple Data Sources

### Using Multi-Input Operations

```javascript
import { Pipeline, intersection, difference, union, merge } from 'orlando-transducers';

// Find users active on both platforms
const mobileUsers = new Pipeline()
  .filter(e => e.platform === 'mobile')
  .map(e => e.userId)
  .toArray(events);

const webUsers = new Pipeline()
  .filter(e => e.platform === 'web')
  .map(e => e.userId)
  .toArray(events);

const crossPlatform = intersection(mobileUsers, webUsers);
const mobileOnly = difference(mobileUsers, webUsers);
const allUsers = union(mobileUsers, webUsers);
```

### Interleaving Data Streams

```javascript
import { merge, Pipeline } from 'orlando-transducers';

// Process logs from multiple servers
const processLogs = new Pipeline()
  .filter(log => log.level === 'error')
  .map(log => ({
    server: log.source,
    message: log.message,
    time: new Date(log.timestamp),
  }));

const server1Errors = processLogs.toArray(server1Logs);
const server2Errors = processLogs.toArray(server2Logs);

// Interleave for chronological review
const allErrors = merge([server1Errors, server2Errors]);
```

## Debugging Pipelines

Use `.tap()` to inspect values flowing through the pipeline without modifying them:

```javascript
const pipeline = new Pipeline()
  .tap(x => console.log('[input]', x))
  .filter(x => x.active)
  .tap(x => console.log('[after filter]', x))
  .map(x => x.email.toLowerCase())
  .tap(x => console.log('[after map]', x))
  .take(5);

const result = pipeline.toArray(users);
```

### Conditional Debugging

```javascript
const DEBUG = process.env.NODE_ENV === 'development';

function debug(label) {
  return DEBUG
    ? x => console.log(`[${label}]`, x)
    : () => {};
}

const pipeline = new Pipeline()
  .tap(debug('raw'))
  .filter(isValid)
  .tap(debug('valid'))
  .map(transform)
  .tap(debug('transformed'));
```

## Rust: PipelineBuilder for ETL

```rust
use orlando_transducers::iter_ext::PipelineBuilder;

// Extract numeric values, filter outliers, take top results
let cleaned: Vec<f64> = PipelineBuilder::new()
    .map(|record: Record| record.value)
    .filter(|v: &f64| *v > 0.0 && *v < 1000.0)
    .take(100)
    .run(raw_records.into_iter());
```

## Rust: Hybrid Composition

```rust
use orlando_transducers::{Map, Filter, Take, to_vec, intersection};

// Process each dataset independently
let pipeline = Map::new(|r: Record| r.user_id)
    .compose(Filter::new(|id: &u64| *id > 0));

let dataset_a_ids = to_vec(&pipeline, dataset_a);
let dataset_b_ids = to_vec(&pipeline, dataset_b);

// Find common users
let common_users = intersection(dataset_a_ids, dataset_b_ids);
```
