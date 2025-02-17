use crate::context::{AgentContext, Context};
use crate::performers::Next;
use crate::runtime::RunAgent;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::{InteractiveTask, ManagedContext, ReachableContext};
use std::any::type_name;

/// `Agent` is a universal trait of a hybrid (transactional) actor.
/// It has a rich lifecycle, beginning its execution with the `initialize` method,
/// which in turn calls the `begin` method. This nesting allows for initialization
/// with context or simply setting an initial state.
///
/// `Agent` is an actor that either reactively processes incoming messages
/// or executes special states, similar to a finite state machine,
/// temporarily blocking message processing. This enables the actor to be prepared
/// for operation or reconfigured in a transactional mode.
#[async_trait]
pub trait Agent: Sized + Send + 'static {
    type Context: AgentContext<Self>;

    /// The `initialize` method is called first when the actor starts.
    /// It should return a `Next` state, which the actor will transition to.
    ///
    /// An execution context is passed as a parameter.
    ///
    /// By default, the method implementation calls the `begin` method.
    fn initialize(&mut self, _ctx: &mut Context<Self>) -> Next<Self> {
        self.begin()
    }

    /// The `begin` method is an initialization method without context.
    ///
    /// It is usually the most commonly used method to start the actor
    /// in transactional mode by initiating finite state machine message processing.
    ///
    /// If the method is not overridden, it starts the actor's mode—reactive
    /// message processing. You can achieve this by calling the `Next::events()` method.
    fn begin(&mut self) -> Next<Self> {
        Next::events()
    }

    /// The method is called when an attempt is made to interrupt the agent's execution.
    ///
    /// It is triggered only in actor mode. If the agent is in finite state machine mode,
    /// it will not be called until it transitions to a reactive state
    /// by invoking `Next::events()`.
    ///
    /// The default implementation interrupts the agent's execution by calling
    /// the `Context::shutdown()` method.
    ///
    /// By overriding this method, you can define your own termination rules,
    /// for example, initiating a finalization process that determines whether
    /// the agent should actually be terminated.
    fn interrupt(&mut self, ctx: &mut Context<Self>) {
        ctx.shutdown();
    }

    /// The method responsible for handling messages in actor mode.
    ///
    /// The default implementation calls the `next_envelope()` method of the agent's context,
    /// thereby retrieving a message with a handler and executing it by calling the `handle()` method.
    ///
    /// If the channel has been closed, the `stop()` method of the context will be called,
    /// immediately halting execution. The request message for interrupting the actor is received
    /// through the same mechanism and closes the channel.
    async fn event(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        let envelope = ctx.next_envelope();
        if let Some(envelope) = envelope.await {
            envelope.handle(self, ctx).await?;
        } else {
            // Terminates the runtime when the channel has drained
            ctx.stop();
        }
        Ok(())
    }

    /// This method is called every time a handler call ends with an unhandled error.
    ///
    /// By default, it simply logs the error, but you can also define additional actions
    /// since the method has access to the agent's context.
    fn failed(&mut self, err: Error, _ctx: &mut Context<Self>) {
        log::error!("Agent [{}] failed: {err}", type_name::<Self>());
    }

    async fn rollback(_this: Option<&mut Self>, _err: Error, _ctx: &mut Context<Self>) {}

    fn finalize(&mut self, _ctx: &mut Context<Self>) {
        self.end()
    }

    fn end(&mut self) {}
}

pub trait Standalone: Agent {
    fn spawn(self) -> <Self::Context as ReachableContext>::Address
    where
        Self::Context: Default,
    {
        RunAgent::new(self).spawn_connected()
    }

    // TODO: spawn_with_context()
}

pub trait Runnable: Agent {
    fn run(self) -> RunAgent<Self>;
}

impl<A: Agent> Runnable for A
where
    Self::Context: Default,
{
    fn run(self) -> RunAgent<Self> {
        RunAgent::new(self)
    }
}
