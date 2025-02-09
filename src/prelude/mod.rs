use std::future::Future;
use tokio::runtime::Handle;
use tokio::task::JoinSet;

pub trait IteratorExt<T> {
    fn to_set(self) -> JoinSet<T>;
}
impl<I, F, T> IteratorExt<T> for I
where
    T: Send + 'static,
    F: Future<Output = T> + Send + 'static,
    I: Iterator<Item = F> + Sized,
{
    fn to_set(self) -> JoinSet<T> {
        JoinSet::from_iter(self)
    }
}

pub trait FutureExt<T> {
    fn await_blocking(self) -> T;
}
impl<T, F: Future<Output = T> + Send> FutureExt<T> for F {
    fn await_blocking(self) -> T {
        tokio::task::block_in_place(|| Handle::current().block_on(self))
    }
}
