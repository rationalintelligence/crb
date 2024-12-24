# CRB | Composable Runtime Blocks

**CRB** is a collection of asynchronous and synchronous blocks for Rust designed to build large modular applications.

The library combines various approaches, including workers, actors, agents, routines, concurrency, parallelism, and pipelines.

The core idea is to ensure all blocks are highly compatible with each other, enabling significant code reuse.

# Key Advantages

## WASM Compatibility

One of the library's major advantages is its out-of-the-box compatibility with WebAssembly (WASM). This allows you to write full-stack solutions in Rust while reusing the same codebase across different environments.

> Note: Synchronous primitives are currently unavailable in WASM due to its lack of full thread support. However, using them in environments like browsers is generally unnecessary, as they block asynchronous operations.

# Using the Library

## Adding a Dependency

Let's start with how to use the library. Although it consists of numerous crates with different types of blocks and foundational components, everything is unified under the main `crb` crate. To add it to your project, simply run:

```bash
cargo add crb
```
