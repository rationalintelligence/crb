<img src="./assets/crb-header.png" width="100%" />

# CRB | Composable Runtime Blocks

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Documentation][docs-badge]][docs-url]

[crates-badge]: https://img.shields.io/crates/v/crb.svg
[crates-url]: https://crates.io/crates/crb
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/runtime-blocks/crb/blob/master/LICENSE
[docs-badge]: https://docs.rs/crb/badge.svg
[docs-url]: https://docs.rs/crb

A unique framework that implementes **hybrid workloads**, seamlessly combining synchronous and asynchronous activities, state machines, routines, the actor model, and supervisors.

It’s perfect for building massive applications and serves as an ideal low-level framework for creating your own frameworks, for example AI-agents.
The core idea is to ensure all blocks are highly compatible with each other, enabling significant code reuse.

# What is a hybrid workload?

A hybrid workload is a concurrent task capable of switching roles - it can function as a synchronous or asynchronous task, a finite state machine, or as an actor exchanging messages.

The implementation is designed as a **fully portable solution** that can run in a standard environment, a WASM virtual machine (e.g., in a browser), or a TEE enclave. This approach significantly reduces development costs by allowing you to reuse code across all parts of your application: backend, frontend, agents, and more.

<img src="./assets/crb-arch.png" width="400px" align="center" />

The key feature is its ability to combine the roles, enabling the implementation of algorithms with **complex branching** that would be impossible in the flat structure of a standard function. This makes it ideal for building the framework of large-scale applications or implementing complex workflows, such as AI pipelines.

<img src="./assets/crb-hybryd.png" width="400px" align="center" />

# Examples

Below, you'll find numerous examples of building hybrid activities using the framework. These examples are functional but simplified for clarity:

- Type imports are omitted to keep examples clean and focused.
- The `async_trait` macro is used but not explicitly shown, as exported traits with asynchronous methods are expected in the future.
- `anyhow::Result` is used instead of the standard `Result` for simplicity.
- Agent context is not specified, as it is always `AgentSession` in these cases.
- The output type `Output` is omitted, as it is always `()` here and may become a default in the future.

The examples demonstrate various combinations of states and modes, but in reality, there are many more possibilities, allowing for any sequence or combination.

### Before diving into the examples...

To create a universal hybrid activity, you need to define a structure and implement the `Agent` trait for it. By default, the agent starts in a reactive actor mode, ready to receive messages and interact with other actors.

However, you can override this behavior by explicitly specifying the agent's initial state in the `begin()` method.

```rust
pub struct Task;

impl Agent for Task {
    fn begin(&mut self) -> Next<Self> {
        Next::do_async(()) // The next state
    }
}
```

The next state is defined using the `Next` object, which provides various methods for controlling the state machine. To perform an asynchronous activity, use the `do_async()` method, passing the new state as a parameter (default is `()`).

Then, implement the `DoAsync` trait for your agent by defining the asynchronous `once()` method:

```rust
impl DoAsync for Task {
    async fn once(&mut self, _: &mut ()) -> Result<Next<Self>> {
        do_something().await?;
        Ok(Next::done())
    }
}
```

The result should specify the next state of the state machine. If you want to terminate the agent, simply return the termination state by calling the `done()` method on the `Next` type.

## Asynchronous tasks

The simplest example is creating a task that performs an asynchronous activity and then terminates.

```rust
pub struct Task;

impl Agent for Task {
    fn begin(&mut self) -> Next<Self> {
        Next::do_async(())
    }
}

impl DoAsync for Task {
    async fn once(&mut self, _: &mut ()) -> Result<Next<Self>> {
        reqwest::get("https://www.rust-lang.org").await?.text().await?;
        Ok(Next::done())
    }
}
```

### Fallbacks

Unlike standard asynchronous activities, you can implement the `fallback()` method to modify the course of actions in case of an error:

```rust
impl DoAsync for Task {
    async fn fallback(&mut self, err: Error) -> Next<Self> {
        log::error!("Can't load a page: {err}. Trying again...");
        Ok(Next::do_async(()))
    }
}
```

The task will now repeatedly enter the same state until the loading process succeeds.

### Repeated routines

The agent already implements a persistent routine in the `repeat()` method, which repeatedly attempts to succeed by calling the `once()` method. To achieve the same effect, we can simply override that method:

```rust
impl DoAsync for Task {
    async fn repeat(&mut self, _: &mut ()) -> Result<Option<Next<Self>>> {
        reqwest::get("https://www.rust-lang.org").await?.text().await?;
        Ok(Some(Next::done()))
    }
}
```

The `repeat()` method will continue to run until it returns the next state for the agent to transition to.

## Synchronous tasks

To implement a synchronous task simply call the `do_sync()` method on `Next`:

```rust
pub struct Task;

impl Agent for Task {
    fn begin(&mut self) -> Next<Self> {
        Next::do_sync(())
    }
}
```

Next, implement the `DoSync` trait to run the task in a thread (either the same or a separate one, depending on the platform):

```rust
impl DoSync for Task {
    fn once(&mut self, _: &mut ()) -> Result<Next<Self>> {
        let result: u64 = (1u64..=20).map(|x| x.pow(10)).sum();
        println!("{result}");
        Ok(Next::done())
    }
}
```

In the example, it calculates the sum of powers and prints the result to the terminal.


## Multiple states (state-machines)

Interestingly, you can define different states and implement unique behavior for each, whether synchronous or asynchronous. This gives you both a state machine and varied execution contexts without the need for manual process management.

Let’s create an agent that prints the content of a webpage to the terminal. The first state the agent should transition to is `GetPage`, which includes the URL of the page to be loaded. This state will be asynchronous, so call the `do_async()` method with the `Next` state.

```rust
pub struct Task;

impl Agent for Task {
    fn begin(&mut self) -> Next<Self> {
        let url = "https://www.rust-lang.org".into();
        Next::do_async(GetPage { url })
    }
}
```

### Asynchronous state

Implement the `GetPage` state by defining the corresponding structure and using it in the `DoAsync` trait implementation for our agent `Task`:

```rust
struct GetPage { url: String }

impl DoAsync<GetPage> for Task {
    async fn once(&mut self, state: &mut GetPage) -> Result<Next<Self>> {
        let text = reqwest::get(state.url).await?.text().await?;
        Ok(Next::do_sync(Print { text }))
    }
}
```

In the `GetPage` state, the webpage will be loaded, and its content will be passed to the next state, `Print`, for printing. Since the next state is synchronous, it is provided as a parameter to the `do_sync()` method.

### Synchronous state

Now, let’s define the `Print` state as a structure and implement the `DoSync` trait for it:

```rust
struct Print { text: String }

impl DoSync<Print> for Task {
    fn once(&mut self, state: &mut Print) -> Result<Next<Self>> {
        printlnt!("{}", state.text);
        Ok(Next::done())
    }
}
```

The result is a state machine with the flexibility to direct its workflow into different states, enabling the implementation of a robust and sometimes highly nonlinear algorithm that would be extremely difficult to achieve within a single asynchronous function.

## Mutable states

Previously, we didn't modify the data stored in a state. However, states provide a convenient context for managing execution flow or collecting statistics **without cluttering the task's main structure**!

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
    async fn repeat(&mut self, mut state: &mut Monitor) -> Result<Option<Next<Self>>> {
        state.total += 1;
        reqwest::get("https://www.rust-lang.org").await?.error_for_status()?;
        state.success += 1;
        sleep(Duration::from_secs(10)).await;
        Ok(None)
    }
}
```

Above is an implementation of a monitor that simply polls a website and counts successful attempts. It does this without modifying the `Task` structure while maintaining access to it.

## Concurrent Task

Within an asynchronous activity, all standard tools for concurrently executing multiple `Futures` are available. For example, in the following code, several web pages are requested simultaneously using the `join_all()` function:

```rust
pub struct ConcurrentTask;

impl Agent for ConcurrentTask {
    fn begin(&mut self) -> Next<Self> {
        Next::do_async(())
    }
}

impl DoAsync for ConcurrentTask {
    async fn once(&mut self, _: &mut ()) -> Result<Next<Self>> {
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

This approach allows for more efficient utilization of the asynchronous runtime while maintaining the workflow without needing to synchronize the retrieval of multiple results.

## Parallel Task

Another option is parallelizing computations. This is easily achieved by implementing a synchronous state. Since it runs in a separate thread, it doesn't block the asynchronous runtime, allowing other agents to continue executing in parallel.

```rust
pub struct ParallelTask;

impl Agent for ParallelTask {
    fn begin(&mut self) -> Next<Self> {
        Next::do_sync(())
    }
}

impl DoSync for ParallelTask {
    fn once(&mut self, _: &mut ()) -> Result<Next<Self>> {
        let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let squares = numbers.into_par_iter().map(|n| n * n).collect();
        Ok(Next::done())
    }
}
```

In the example above, parallel computations are performed using the `rayon` crate. The results are awaited asynchronously by the agent since `DoSync` shifts execution to a thread while continuing to wait for the result asynchronously.

## Subtask Execution

The framework allows tasks to be reused within other tasks, offering great flexibility in structuring code.

```rust
pub struct RunBoth;

impl Agent for RunBoth {
    fn begin(&mut self) -> Next<Self> {
        Next::do_async(())
    }
}

impl DoAsync for RunBoth {
    async fn once(&mut self, _: &mut ()) -> Result<Next<Self>> {
        join!(
            RunAgent::new(ConcurrentTask).run(),
            RunAgent::new(ParallelTask).run(),
        ).await;
        Ok(Next::done())
    }
}
```

The code example implementes an agent that waits for the simultaneous completion of two tasks we implemented earlier: **concurrent** and **parallel**.


## Shared state in a state machine

Although the states within a group inherently form a state machine, you can define it more explicitly by adding a field to the agent and describing the states with a dedicated `enum`:

```rust
enum State {
    First,
    Second,
    Third,
}

struct Task {
    state: State,
}
```

In this case, the `State` enumeration can handle transitions between states. Transition rules can be implemented as a function that returns a `Next` instance with the appropriate state handler.

```rust
impl State {
    fn next(&self) -> Next<Task> {
        match self {
            State::First => Next::do_async(First),
            State::Second => Next::do_async(Second),
            State::Third => Next::do_async(Third),
        }
    }
}
```

Set the initial state when creating the task (it can even be set in the constructor), and delegate state transitions to the `next()` function, called from the `begin()` method.

```rust
impl Agent for Task {
    fn begin(&mut self) -> Next<Self> {
        self.state.next()
    }
}
```

Implement all state handlers, with the `State` field determining the transition to the appropriate handler. Simply use its `next()` method so that whenever the state changes, the transition always leads to the correct handler.

```rust
struct First;

impl DoAsync<First> for Task {
    async fn once(&mut self, _: &mut First) -> Result<Next<Self>> {
        self.state = State::Second;
        Ok(self.state.next())
    }
}

struct Second;

impl DoAsync<Second> for Task {
    async fn once(&mut self, _: &mut Second) -> Result<Next<Self>> {
        self.state = State::Third;
        Ok(self.state.next())
    }
}

struct Third;

impl DoAsync<Third> for Task {
    async fn once(&mut self, _: &mut Third) -> Result<Next<Self>> {
        Ok(Next::done())
    }
}
```

💡 The framework is so flexible that it allows you to make this logic even more explicit by implementing a custom `Performer` for the agent.

## Actor Model

Agents handle messages asynchronously. Actor behavior is enabled by default if no transition state is specified in the `begin()` method. Alternatively, you can switch to this mode from any state by calling `Next::process()`, which starts message processing.

In other words, the actor state is the default for the agent, so simply implementing the `Agent` trait is enough:

```rust
struct Actor;

impl Agent for Actor {}
```

An actor can accept any data type as a message, as long as it implements the `OnEvent` trait for that type and its `handle()` method. For example, let's teach our actor to accept an `AddUrl` message, which adds an external resource:

```rust
struct AddUrl { url: Url }

impl OnEvent<AddUrl> for Actor {
    async handle(&mut self, event: AddUrl, ctx: &mut Self::Context) -> Result<()> {
        todo!()
    }
}
```

Actors can implement handlers for any number of messages, allowing you to add as many `OnEvent` implementations as needed:

```rust
struct DeleteUrl { url: Url }

impl OnEvent<DeleteUrl> for Actor {
    async handle(&mut self, event: DeleteUrl, ctx: &mut Self::Context) -> Result<()> {
        todo!()
    }
}
```

The provided context (`ctx`) allows you to send a message, terminate the actor, or transition to a new state by setting the next state with `Next`.

> State transitions have the highest priority. Even if there are messages in the queue, the state transition will occur first, and messages will wait until the agent returns to the actor state.

## Interactions

The actor model is designed to let you add custom handler traits for any type of event. For example, this framework supports interactive actor interactions—special messages that include a request and a channel for sending a response.

The example below implements a server that reserves an `Id` in response to an interactive request and returns it:

```rust
struct Server {
    slab: Slab<Record>,
}

struct GetId;

impl Request for GetId {
    type Response = usize;
}

impl OnRequest<GetId> for Server {
    async on_request(&mut self, _: GetId, ctx: &mut Self::Context) -> Result<usize> {
        let record = Record { ... };
        Ok(self.slab.insert(record))
    }
}
```

The request must implement the `Request` trait to specify the `Response` type. As you may have noticed, for the `OnRequest` trait, we implemented the `on_request()` method, which expects a response as the result. This eliminates the need to send it manually.

The following code implements a `Client` that configures itself by sending a request to the server to obtain an `Id` and waits for a response by implementing the `OnResponse` trait.

```rust
struct Client {
    server: Address<Server>,
}

impl Agent for Client {
    fn begin(&mut self) -> Next<Self> {
        Next::in_context(Configure)
    }
}

struct Configure;

impl InContext<Configure> for Client {
    async fn once(&mut self, _: &mut Configure, ctx: &mut Self::Context) -> Result<Next<Self>> {
        self.server.request(GetId)?.forward_to(ctx)?;
        Ok(Next::process())
    }
}

impl OnResponse<GetId> for Client {
    async on_response(&mut self, id: usize, ctx: &mut Self::Context) -> Result<()> {
        println!("Reserved id: {id}");
        Ok(())
    }
}
```

## Supervisor

An actor (or any agent) can be launched from anywhere if it implements the `Standalone` trait. Otherwise, it can only be started within the context of a supervisor.

The supervisor can also manage all spawned tasks and terminate them in a specific order by implementing the `Supervisor` trait:

```rust
struct App;

impl Agent for App {
    type Context = SupervisorSession<Self>;
}

impl Supervior for App {
    type Group = Group;
}
```

The `Group` type defines a grouping for tasks, where the order of its values determines the sequence in which child agents (tasks, actors) are terminated.

```rust
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Group {
    Workers,
    Requests,
    Server,
    HealthCheck,
    DbCleaner,
    UserSession(Uid),
}
```

## Pipelines

The framework includes an experimental implementation of pipelines that automatically trigger tasks as they process input data from the previous stage.

However, creating complex workflows is also possible using just the agent's core implementation.

## Functional activities (`fn` or `Future` as tasks)

> todo

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

# Author

The project was originally created by [@therustmonk](https://github.com/therustmonk) as a result of extensive experimental research into implementing a hybrid actor model in Rust.

<a href="https://crateful.substack.com/" target="_blank"><img src="./assets/crateful-logo.png" width="100px" /></a>

To support the project, please subscribe to [Crateful](https://crateful.substack.com/), my newsletter about Rust crates, which gathers information using a cast of crab agents written in this framework.

# License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/runtime-blocks/crb/blob/master/LICENSE
