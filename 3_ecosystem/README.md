Step 3: Common ecosystem
========================

__Estimated time__: 2 days

These steps describe common crates and tools in [Rust] ecosystem required for application and library development.

> ❗️Before completing this step you should complete all its sub-steps.

After doing them you should be able to answer the following questions:

- What testing capabilities does [Rust] offer and when should I use them? Why should I follow [BDD] style?

  Rust offers unit tests (inside modules with `#[test]`), integration tests (in `tests/` directory), doc tests (in `///` comments), and benchmarks (`#[bench]`, nightly). Unit tests are for isolated logic, integration tests for public API behavior, doc tests to keep examples correct. BDD style (`given/when/then` naming) makes test intent clear and serves as living documentation.

- What are macros? How do they differ? What benefits does their usage give? When should I write one?

  Macros are code that generates code at compile time. Declarative macros (`macro_rules!`) match patterns and expand to code — good for repetitive syntax. Procedural macros (`#[derive]`, attribute, function-like) operate on the AST and are more powerful. Benefits: reduce boilerplate, enable DSLs, enforce patterns. Write one when you need to generate repetitive code that can't be abstracted with functions or traits.

- How to work with date and time in [Rust]? How should I store time? How should I return it to other applications?

  Use the `chrono` or `time` crate. Store time as UTC (`DateTime<Utc>`) internally — never local time. Return it to other applications as ISO 8601 string (`2024-01-01T00:00:00Z`) or Unix timestamp (seconds since epoch). Avoid storing local time as it's ambiguous across timezones.

- How are regular expressions used in [Rust]? When are they not enough? How can I write a custom parser in [Rust]?

  Use the `regex` crate — compile once with `lazy_static!` or `OnceLock`, reuse many times. Regexes are not enough for nested/recursive grammars (e.g. JSON, HTML, programming languages). For custom parsers use `nom` (parser combinators) or `pest` (PEG grammars) — both are composable and produce typed results.

- How do iterator and collection compare and differ in [Rust]? What is the purpose of immutable collections? Why should I care about concurrent collections?

  Collections store data, iterators lazily process it. Iterators are zero-cost — they compile to the same code as hand-written loops. Immutable collections (e.g. from `im` crate) allow safe structural sharing — useful in functional patterns and undo/redo. Concurrent collections (e.g. `DashMap`) allow safe shared access from multiple threads without wrapping everything in `Mutex`.

- What should I use for serialization in [Rust]? Why this is good or bad?

  `serde` is the standard — it's a framework, not an implementation. Formats (`serde_json`, `serde_toml`, etc.) plug into it. Good: zero-cost, derive-based, format-agnostic. Bad: complex error messages, some edge cases with lifetimes and `#[serde(flatten)]` can be tricky.

- How can I generate randomness in [Rust]? Which guarantees of random generator should I choose and when?

  Use the `rand` crate. For general use: `rand::rng()` (cryptographically seeded, fast). For cryptography: `rand::rngs::OsRng` (OS entropy). For reproducible tests: `StdRng::seed_from_u64(42)`. Never use a fast PRNG for security-sensitive code.

- What should I use for password hashing in [Rust]? How can I encrypt a message? How should I compare secret values?

  For password hashing: `argon2`, `bcrypt`, or `scrypt` — they're slow by design. For encryption: `ring` or `rustls`. For comparing secrets: always use constant-time comparison (`subtle::ConstantTimeEq`) to prevent timing attacks — never `==`.

- How logging is organized in [Rust] ecosystem? Why should I care about structured logging?

  `log` crate defines the facade (`info!`, `warn!`, etc.), implementations like `env_logger` or `tracing-subscriber` handle output. `tracing` is the modern choice — it supports structured fields, spans, and async context. Structured logging (key-value pairs instead of plain strings) makes logs machine-parseable and filterable in tools like Grafana/Loki.

- What should I use for building [CLI] interface in [Rust]? How can I organize a configuration for my application and why?

  Use `clap` with derive feature for CLI. For configuration: `config` crate merges multiple sources (defaults in code → file → env vars) with priority. This follows the 12-factor app principle — config is separated from code, secrets stay out of the repo.

- Why multithreading is required for [Rust] programs and what problems does it solve? How threads concurrency differs with parallelism? How can I parallelize code in [Rust]?

  Threads allow utilizing multiple CPU cores (parallelism) and handling concurrent I/O without blocking. Concurrency is about structure (tasks overlap in time), parallelism is about execution (tasks run simultaneously). For parallelism use `rayon` — `par_iter()` turns sequential iterators parallel with minimal code change.

- What is asynchronicity and what problems does it solve? How is it compared to threads concurrency? What is [Rust] solution for asynchronicity and why it has such design?

  Async solves I/O-bound problems: instead of one thread per connection, one thread handles thousands of concurrent I/O operations. Threads are better for CPU-bound work. Rust's solution is poll-based `Future` — lazy, zero-cost (compiles to state machines), driven by a runtime like `tokio`. Poll-based design avoids implicit allocation and gives precise control over execution.

- What are actors? When are they useful?

  Actors are isolated units of state that communicate only via messages (mailbox/channel). No shared state — no data races. Useful in Rust for long-lived stateful entities: WebSocket connections, background workers, anything where you want to encapsulate mutable state behind a message-passing interface. Main implementation: `actix`.




## Task

Write a [CLI] tool for stripping [JPEG] images [metadata][21] and minimizing their size (a simplified analogue of [tinyjpg.com]).

Requirements:
- Accept input list of files and remote [URL]s via: either [CLI] arguments, [STDIN], or read it from a specified file ([EOL]-separated).
- Allow configuring how much images are processed at the same time.
- Allow configuring the output directory to store processed images in.
- Allow configuring the output [JPEG] quality of processed images.
- Read configuration with ascending priority from: a file (format is on your choice), [environment variables][22], [CLI] arguments. All are optional for specifying.
- Support `RUST_LOG` environment variable, allowing granular tuning of log levels per module.
- Print execution time in logs, so it's easy to see how much which operation takes during the execution.

If you have enough time after implementing base requirements, consider to add the following to your solution:
- Allow configuring download speed limit for images from remote [URL]s.
- Cover your implementation with unit and E2E tests.
- Support [PNG] images as well.
- Add comprehensive documentation to your code.




[BDD]: https://en.wikipedia.org/wiki/Behavior-driven_development
[CLI]: https://en.wikipedia.org/wiki/Command-line_interface
[EOL]: https://en.wikipedia.org/wiki/Newline
[JPEG]: https://en.wikipedia.org/wiki/JPEG
[PNG]: https://en.wikipedia.org/wiki/PNG
[Rust]: https://www.rust-lang.org
[STDIN]: https://en.wikipedia.org/wiki/Standard_streams#Standard_input_(stdin)
[tinyjpg.com]: https://tinyjpg.com
[URL]: https://en.wikipedia.org/wiki/URL

[21]: https://picvario.com/what-is-image-metadata-role-and-benefits
[22]: https://en.wikipedia.org/wiki/Environment_variable
