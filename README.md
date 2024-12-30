<img src="./assets/crb-header.png" width="100%" />

# CRB | Composable Runtime Blocks

A unique framework that implementes **hybrid workloads**, seamlessly combining synchronous and asynchronous activities, state machines, routines, the actor model, and supervisors.

It’s perfect for building massive applications and serves as an ideal low-level framework for creating your own frameworks, for example AI-agents.
The core idea is to ensure all blocks are highly compatible with each other, enabling significant code reuse.

<a href="https://crateful.substack.com/" target="_blank"><img src="./assets/crateful-logo.png" width="100px" /></a>

I created this project to build an free AI-curated Rust magazine called [Crateful](https://crateful.substack.com/), written entirely in Rust.

# Examples

Code examples are simplified!

## Async Task

### Single-state single-run `async` task

```rust
pub struct Task;

impl Agent for Task {
    fn begin(&mut self) -> Next<Self> {
        Next::do_async(())
    }
}

impl DoAsync for Task {
    async fn once(&mut self, _: ()) -> Result<Next<Self>> {
        reqwest::get("https://www.rust-lang.org").await?.text().await?;
        Ok(Next::done())
    }
}
```

### Multi-state single-run `async` than `sync` task

```rust
pub struct Task;

impl Agent for Task {
    fn begin(&mut self) -> Next<Self> {
        let url = "https://www.rust-lang.org".into();
        Next::do_async(GetPage { url })
    }
}

struct GetPage { url: String }

impl DoAsync<GetPage> for Task {
    async fn once(&mut self, state: GetPage) -> Result<Next<Self>> {
        let text = reqwest::get(state.url).await?.text().await?;
        Ok(Next::do_sync(Print { text }))
    }
}

struct Print { text: String }

impl DoSync<Print> for Task {
    fn once(&mut self, state: Print) -> Result<Next<Self>> {
        printlnt!("{}", state.text);
        Ok(Next::done())
    }
}
```

### Repetetive async task

```rust
pub struct Task;

impl Agent for Task {
    fn begin(&mut self) -> Next<Self> {
        Next::do_async(Monitor)
    }
}

struct Monitor {
    total: u64,
    success: u64,
}

impl DoAsync<Monitor> for Task {
    async fn repeat(&mut self, mut state: Monitor) -> Result<Option<Next<Self>>> {
        state.total += 1;
        reqwest::get("https://www.rust-lang.org").await?.error_for_status()?;
        state.success += 1;
        sleep(Duration::from_secs(10)).await;
        Ok(None)
    }
}
```

### Concurrent Task

```rust
pub struct ConcurrentTask;

impl Agent for ConcurrentTask {
    fn begin(&mut self) -> Next<Self> {
        Next::do_async(())
    }
}

impl DoAsync for ConcurrentTask {
    async fn once(&mut self, _: ()) -> Result<Next<Self>> {
        let urls = vec![
            "https://www.rust-lang.org",
            "https://www.crates.io",
            "https://crateful.substack.com",
            "https://knowledge.dev",
        ];
        let futures = urls.into_iter().map(|url| reqwest::get(url));
        future::join_all(futures).await
        Ok(Next::done())
    }
}
```

### Parallel Task

```rust
pub struct ParallelTask;

impl Agent for ParallelTask {
    fn begin(&mut self) -> Next<Self> {
        Next::do_sync(())
    }
}

impl DoSync for ParallelTask {
    fn once(&mut self, _: ()) -> Result<Next<Self>> {
        let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let squares = numbers.into_par_iter().map(|n| n * n).collect();
        Ok(Next::done())
    }
}
```

### Task split

```rust
pub struct RunBoth;

impl Agent for RunBoth {
    fn begin(&mut self) -> Next<Self> {
        Next::do_async(())
    }
}

impl DoAsync for RunBoth {
    async fn once(&mut self, _: ()) -> Result<Next<Self>> {
        join!(
            RunAgent::new(ConcurrentTask).run(),
            RunAgent::new(ParallelTask).run(),
        ).await;
        Ok(Next::done())
    }
}
```

## State Machine

```rust
pub struct Fsm;

impl Agent for Fsm {
    fn begin(&mut self) -> Next<Self> {
        Next::do_async(StateOne)
    }
}

struct StateOne;

impl DoAsync<StateOne> for Fsm {
    async fn once(&mut self, _: StateOne) -> Result<Next<Self>> {
        Ok(Next::do_async(StateTwo))
    }
}

struct StateTwo;

impl DoAsync<StateTwo> for Fsm {
    async fn once(&mut self, _: StateTwo) -> Result<Next<Self>> {
        Ok(Next::do_async(StateThree::default()))
    }
}

#[derive(Default)]
struct StateThree { counter: u64 }

impl DoAsync<StateThree> for Fsm {
    async fn once(&mut self, mut state: StateThree) -> Result<Next<Self>> {
        state.counter += 1;
        Ok(Next::do_async(state))
    }
}
```

## Actor Model

## Agent | Hybryd Actor

## Compatibility

### Threads (`async+sync` tasks) support

```toml
[dependencies]
crb-agent = { version = "0.0.21", features = ["sync"] }
```

### WASM mode (`async` tasks only)

```toml
[dependencies]
crb-agent = { version = "0.0.21", default-features = false }
```

# Key Advantages

## WASM Compatibility

One of the library's major advantages is its out-of-the-box compatibility with WebAssembly (WASM). This allows you to write full-stack solutions in Rust while reusing the same codebase across different environments.

> Synchronous tasks are currently unavailable in WASM due to its lack of full thread support. However, using them in environments like browsers is generally unnecessary, as they block asynchronous operations.

## Actor Model

The library includes a complete implementation of the actor model, enabling you to build a hierarchy of actors and facilitate message passing between them. When the application stops, actors gracefully shut down between messages processing phases, and in the specified order.

## Synchronous Tasks

The framework supports not only asynchronous activities (IO-bound) but also allows running synchronous (CPU-bound) tasks using threads. The results of these tasks can seamlessly be consumed by asynchronous activities.

## Pipelines

The library offers a Pipeline implementation compatible with actors, routines, and tasks (including synchronous ones), making it ideal for building AI processing workflows.

## Trait-Based Design

Unlike many actor frameworks, this library relies heavily on traits. For example, tasks like interactive communication, message handling, or `Stream` processing are implemented through specific trait implementations.

More importantly, the library is designed to be extensible, allowing you to define your own traits for various needs while keeping your code modular and elegant. For instance, actor interruption is implemented on top of this model.

## Method Hierarchy

Trait methods are designed and implemented so that you only need to define specific methods to achieve the desired behavior.

Alternatively, you can fully override the behavior and method call order - for instance, customizing an actor’s behavior in any way you like or adding your own intermediate phases and methods.

## Error Handling and Resilience

The library provides built-in error handling features, such as managing failures during message processing, making it easier to write robust and resilient applications.

# Using the Library

## Adding a Dependency

Let's start with how to use the library. Although it consists of numerous crates with different types of blocks and foundational components, everything is unified under the main `crb` crate. To add it to your project, simply run:

```bash
cargo add crb
```
