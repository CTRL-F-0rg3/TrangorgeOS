use core::marker::PhantomData;

pub struct SendSync<T>(PhantomData<T>);
unsafe impl<T> Send for SendSync<T> {}
unsafe impl<T> Sync for SendSync<T> {}

pub struct Immutable<T>(PhantomData<T>);
impl<T> Immutable<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

pub struct Unpin<T>(PhantomData<T>);
impl<T> core::marker::Unpin for Unpin<T> {}