# CRB v0.0.23 - 2025-01-04

## Improvements

- **Hybryd actors** - Stabilized task transitions between state-machine states and actor states and vice versa.
- **Hybryd runtimes** - Debugged the ability of the state-machine to execute synchronous `DoSync` and asynchronous `DoAsync` blocks in any sequence, or switching to actor mode.

## Added

- **Finalizers** - Added actor finalizers, which enable running callbacks when an actor finishes. This simplifies the process for a supervisor to receive notifications upon actor termination.
- **Error tracking method** - Added a `failed()` method to the `Agent` trait, which is called for each unhandled error in either the actor's handlers or the state machine's handlers.
- **Awaiting agents in-place** - Added a separate `Runnable` trait that allows any hybrid task (`Agent`) to be executed as a `Future` and to obtain the result.
- **Repair handling** - Added a `repair()` method to the `DoAsync` and `DoSync` states, enabling task recovery in case of an error.
- **Access to an address** - Added capability for `AgentSession` context to access its own `Address` through the implementation of the `Deref` trait.
