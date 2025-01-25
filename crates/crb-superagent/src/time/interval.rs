use super::timer::Timer;
use crb_agent::{OnEvent, ToAddress};
use crb_core::{time::Duration, Tag};
use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut)]
pub struct Interval<T> {
    timer: Timer<T>,
}

impl<T> Interval<T>
where
    T: Tag + Clone,
{
    pub fn new(event: T, duration: Duration) -> Self {
        let mut timer = Timer::new(event);
        timer.set_duration(duration);
        timer.set_repeat(true);
        Self { timer }
    }

    pub fn enable<A>(&mut self, address: impl ToAddress<A>)
    where
        A: OnEvent<T>,
    {
        self.timer.add_listener(address);
    }
}

impl<T> Default for Interval<T>
where
    T: Tag + Clone + Default,
{
    fn default() -> Self {
        Self::new(T::default(), Duration::from_secs(1))
    }
}
