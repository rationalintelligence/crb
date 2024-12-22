pub trait Message: Send + 'static {}

impl<M> Message for M where M: Send + 'static {}
