# CRB | Composable Runtime Blocks

**CRB** is a collection of asynchronous and synchronous blocks for Rust designed to build large modular applications.

The library combines various approaches, including workers, actors, agents, routines, concurrency, parallelism, and pipelines.

The core idea is to ensure all blocks are highly compatible with each other, enabling significant code reuse.

<a href="https://crateful.substack.com/" target="_blank"><img src="./assets/crateful-logo.png" width="100" /></a>

I created this project to build an free AI-curated Rust magazine called [Crateful](https://crateful.substack.com/), written entirely in Rust.

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
