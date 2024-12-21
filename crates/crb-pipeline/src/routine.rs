pub trait ConductedRoutine {
    type Input: Send;
    type Output: Clone + Sync + Send;

    fn input(input: Self::Input) -> Self;
    fn output(&mut self) -> Self::Output;
}
