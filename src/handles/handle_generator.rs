use super::Handle;
use std::marker::PhantomData;
pub struct HandleGenerator<T> {
    counter: u32,
    handle_type: PhantomData<T>,
}

impl<T> HandleGenerator<T> {
    pub(crate) fn new() -> Self {
        Self {
            counter: 0,
            handle_type: PhantomData,
        }
    }

    pub(crate) fn next_handle(&mut self) -> Handle<T> {
        let out = Handle::<T>(self.counter, PhantomData);
        self.counter += 1;
        out
    }
}
