use core::future::Future;
use futures::stream::FuturesUnordered;
use tokio::task::JoinHandle;

#[derive(Default)]
pub(super) struct TaskPool(FuturesUnordered<JoinHandle<()>>);

impl TaskPool {
    pub(super) fn spawn_task<Fut>(&self, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.0.push(tokio::spawn(future));
    }
}

impl Drop for TaskPool {
    fn drop(&mut self) {
        core::mem::replace(&mut self.0, FuturesUnordered::new())
            .into_iter()
            .for_each(|future| future.abort());
    }
}
