# Changelog

All notable changes to Orlando will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-01-07

### Added

#### Phase 6a: Functional Optics (Lenses)

**Core Optics:**
- `Lens<S, A>` - Total focus on nested data with get/set/over operations
- `Optional<S, A>` - Partial focus for nullable fields with safe None handling
- Lens composition via `compose()` for deep nested access
- All three lens laws verified via property-based tests (GetPut, PutGet, PutPut)

**JavaScript API:**
- `lens(property)` - Create a lens focusing on an object property
- `lensPath(path)` - Create a lens for deep nested paths via arrays
- `optional(property)` - Create an optional lens for nullable fields
- `JsLens` methods: `get()`, `set()`, `over()`, `compose()`
- `JsOptional` methods: `get()`, `getOr()`, `set()`, `over()`

**Unique Feature - Streaming Lenses:**
- **First lens library to integrate with transducers** for streaming data processing
- Extract nested values with lenses, transform with transducers in single pass
- Bounded memory processing of large datasets
- Combines functional optics with high-performance streaming

**Testing:**
- 24 new unit tests for Rust lens operations
- 12 property-based tests verifying lens laws automatically
- 14 comprehensive WASM tests for JavaScript API
- Lens composition correctness tests
- Optional Some/None behavior tests

**Documentation:**
- Comprehensive Phase 6 implementation plan
- Competitive analysis of existing lens libraries
- Lens laws documentation with examples
- Streaming lens integration examples
- Real-world use cases (React/Redux state updates, bulk transformations)

#### Infrastructure
- Total tests: 243 (229 unit + 127 property + 64 integration + 111 doc)
- Type aliases for clippy compliance
- Full TypeScript definitions auto-generated

### Changed
- Updated README.md with comprehensive optics section
- Enhanced project structure documentation
- Added optics to API reference tables

## [0.3.0] - 2025-01-24

### Added
- CI/CD pipeline for automated npm publishing
- Comprehensive publishing guide (PUBLISHING.md)
- .npmignore for npm package optimization

### Changed
- Updated repository URLs to actual GitHub repository

## [0.1.0] - 2025-01-23

### Added

#### Phase 1: Core Transducers (10 operations)
- `FlatMap` transducer for transforming and flattening nested structures
- `Partition` collector for splitting data into pass/fail groups
- `Find` collector for early-termination searches
- `Reject` transducer as inverse of filter
- `Chunk` transducer for batching elements
- `GroupBy` collector for aggregating by key function
- `None` collector as inverse of some
- `Contains` collector for membership testing
- `Zip`/`ZipWith` collectors for combining arrays
- JavaScript `pluck` helper for property extraction

#### Phase 2a: Multi-Input Operations (6 operations)
- `Merge` helper for round-robin interleaving
- `Intersection` helper for set intersection
- `Difference` helper for set difference
- `Union` helper for set union
- `SymmetricDifference` helper for XOR operations
- Hybrid composition pattern documentation

#### Phase 2b: Advanced Collectors (8 operations)
- `CartesianProduct` for generating all possible pairs
- `TopK` for efficient top-N queries
- `ReservoirSample` for uniform random sampling
- `PartitionBy` for consecutive grouping
- `Frequencies` for counting occurrences
- `ZipLongest` for combining arrays with fill values
- `Interpose` transducer (RepeatEach) for element repetition
- `Unique`/`UniqueBy` transducers for deduplication

#### Phase 3: Logic Functions (8 operations)
- `both` predicate combinator (AND logic)
- `either` predicate combinator (OR logic)
- `complement` predicate combinator (NOT logic)
- `allPass` combinator for multiple AND conditions
- `anyPass` combinator for multiple OR conditions
- `When` conditional transducer
- `Unless` conditional transducer
- `IfElse` branching transducer

#### Documentation & Examples
- Complete JavaScript/TypeScript API documentation
- API expansion roadmap
- Hybrid composition guide
- Advanced collectors HTML examples
- Logic functions HTML examples
- Performance benchmarks
- Library comparison benchmarks

#### Testing
- 328 total tests
  - 92 unit tests
  - 64 integration tests
  - 107 property-based tests
  - 65 documentation tests
- Property tests verifying mathematical laws
- Comprehensive integration test coverage

### Performance
- Zero intermediate allocations
- Single-pass execution
- Early termination support
- WASM SIMD optimizations for numeric data
- 3-19x faster than native JavaScript arrays

### Infrastructure
- Git hooks for pre-commit and pre-push checks
- Automated formatting (rustfmt)
- Automated linting (clippy)
- Multi-platform CI testing (Ubuntu, macOS, Windows)
- Code coverage reporting
- WASM test suite

[Unreleased]: https://github.com/justinelliottcobb/Orlando/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/justinelliottcobb/Orlando/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/justinelliottcobb/Orlando/compare/v0.1.0...v0.3.0
[0.1.0]: https://github.com/justinelliottcobb/Orlando/releases/tag/v0.1.0
