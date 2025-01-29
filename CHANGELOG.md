# TODO

- Epochs example
- Child expiration example

# CRB v0.0.28 - upcoming...

## In progress

- Signals listening
- Gracefull shutdown
- Add `repeat_async` and `repeat_sync` Fut/Fn performers
- Use repeaters for the `Drainer`

## Added

- **Interruption levels** - Allow to interrupt activities using a partcular strategy.
- **Main** - A trait to run an agent as a program's entrypoint. Interrupts an agent with levels.
- **Supervisor scheduling** - Schedule an actor spawning.
- **Drainer** - A special task-agent in the superagent crate to drain events sources.
- **ToRecipient** - A trait to produce a recipient for events.
- **Ping** - A ping extension has been added.
- **Operable** - A trait to execute a `Mission` to get the `Goal` result.
- **ForwardTo** - A special trait to assign tasks to the actor itself.
- **Stacker** - A pool for delayed spawning agents.

## Improved

- **Interruptor** - Runtimes interruption was significantly improves. It has some issues that were fixed.
- **Timer** - Both `Timeout` and `Interval` joined into a single worker `Timer`.
- **Unique** - `UniqueId` has been renamed to `Unique`.

# CRB v0.0.27 - 2025-01-18

## Added

- **Timeout events** - Introduced timeout functionality that triggers an event exactly once after the specified duration.
- **Subscriptions pattern** - Implemented a new subscription-based pattern to simplify event handling and improve modularity.
- **Context wrapper** - Added a versatile `Context` wrapper that can now be extended as an `Address`.
- **ToAddress trait** - The new `ToAddress` trait, which allows seamless cloning of addresses from `Address` or `Context` references.
- **Switches** - Added `TimeoutSwitch` and `IntervalSwitch`, reusable components for reusing `Timeout` and `Interval` functionality more efficiently.

## Improved

- **Context renaming** - Renamed the `Context` trait to `ReachableContext` to better convey its purpose and usage.
- **InContext renaming** - Changed the `InContext` trait to `Duty`, reflecting its role in maintaining an agent and its responsibilities.
- **EventSender renaming** - Renamed the `EventSender` struct to `Recipient` (`MessageSender` between the changes) to reduce ambiguity and align with its primary purpose of sending messages.
- **Output removal** - Removed the associated type `Output`. The new `mission` module is now the recommended approach for extracting results from an actor.
- **Slot naming conventions** - Enhanced the naming scheme for slots to improve clarity and consistency across the codebase.



# CRB v0.0.26 - 2025-01-11

## Added

- **TUI example** - Actor-driven TUI [app](https://github.com/runtime-blocks/crb/blob/trunk/examples/tui-app/src/app.rs) that renders the UI in a separated synchronous state.
- **Intervals** - Enables actors to receive notifications at regular intervals.
- **Tagging responses** - Interaction responses can now be tagged, allowing actors to handle responses in various ways.
- **Supervisor example** - Example of a supervisor that respawns a child actor or self-terminates after a child agent fails for the second time.

## Improved

- **Fetcher and Responder** - Introduced more intuitive names for interaction components.
- **Equip for tuples** - The `Equip` trait is now implemented for tuples with an address, simplifying links for spawned agents.

# CRB v0.0.25 - 2025-01-09

## Added

- **Support equipments** - With the `equip()` method call an `Address` could be wrapped with a struct.

## Improved

- **InContext** - `InContext` handle is more effective and doesn't wrap a command into an extra boxed event.



# CRB v0.0.24 - 2025-01-07

## Added

- **Actor and task morphing** - Implemented morphing (molting) from one form (structure) to another; details in
the [example](https://github.com/runtime-blocks/crb/blob/trunk/crates/crb/tests/test_molting.rs).

## Improved

- **Projects list** - Added a list of [projects](https://github.com/runtime-blocks/crb/tree/trunk?tab=readme-ov-file#projects)
that are built using the framework.
- **Consumption** - The actor structure can be completely consumed to produce `Output`.
- **Relaxed requirements** - The return type no longer requires `Default` and `Clone` implementations for the type.
- **Efficient finalizers** - Finalizers use references to `Output`.



# CRB v0.0.23 - 2025-01-04

## Added

- **Finalizers** - Added actor finalizers, which enable running callbacks when an actor finishes. This simplifies the process for a supervisor to receive notifications upon actor termination.
- **Error tracking method** - Added a `failed()` method to the `Agent` trait, which is called for each unhandled error in either the actor's handlers or the state machine's handlers.
- **Awaiting agents in-place** - Added a separate `Runnable` trait that allows any hybrid task (`Agent`) to be executed as a `Future` and to obtain the result.
- **Repair handling** - Added a `repair()` method to the `DoAsync` and `DoSync` states, enabling task recovery in case of an error.
- **Access to an address** - Added capability for `AgentSession` context to access its own `Address` through the implementation of the `Deref` trait.

## Improved

- **Hybryd actors** - Stabilized task transitions between state-machine states and actor states and vice versa.
- **Hybryd runtimes** - Debugged the ability of the state-machine to execute synchronous `DoSync` and asynchronous `DoAsync` blocks in any sequence, or switching to actor mode.
