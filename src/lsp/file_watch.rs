use notify::{Config, Error, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::sync::mpsc::{channel, Receiver};

pub fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (tx, rx) = channel::<notify::Result<Event>>(1);
    let watcher = RecommendedWatcher::new(
        move |res| {
            let tx = tx.clone();
            if let Err(err) = tx.try_send(res) {
                eprintln!("Error sending event: {:?}", err);
            }
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

pub async fn async_watch<P, F, Fut>(path: P, f: F) -> notify::Result<()>
where
    P: AsRef<Path>,
    F: Fn(Result<Event, Error>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let (mut watcher, mut rx) = async_watcher()?;

    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.recv().await {
        f(res).await;
    }

    Ok(())
}
