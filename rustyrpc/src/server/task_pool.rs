use core::future::Future;
use futures::stream::FuturesUnordered;
use tokio::{sync::Mutex, task::JoinHandle};

#[derive(Default)]
pub(super) struct TaskPool(Mutex<FuturesUnordered<JoinHandle<()>>>);

impl TaskPool {
    pub(super) async fn spawn_task<Fut>(&self, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.0.lock().await.push(tokio::spawn(future));
    }
}

impl Drop for TaskPool {
    fn drop(&mut self) {
        tokio::runtime::Handle::current().block_on(async move {
            let mut futures = self.0.lock().await;
            core::mem::replace(&mut *futures, FuturesUnordered::new())
                .into_iter()
                .for_each(|future| future.abort());
        });
    }
}
