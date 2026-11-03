//! Spike: open N TS3 client connections from one IP at once (puppet scenario) and hold them.
use anyhow::Result;
use futures::prelude::*;
use tokio::time::{self, Duration};
use tsclientlib::{Connection, StreamItem};

#[tokio::main]
async fn main() -> Result<()> {
    let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(12);
    let tasks = (0..n).map(|i| async move {
        let mut con = Connection::build("localhost").name(format!("puppet-{i}")).connect()?;
        con.events().try_filter(|e| future::ready(matches!(e, StreamItem::BookEvents(_)))).next().await.transpose()?;
        let mut ev = con.events().try_filter(|_| future::ready(false));
        tokio::select! {
            _ = time::sleep(Duration::from_secs(15)) => Ok::<_, anyhow::Error>(()),
            e = ev.next() => anyhow::bail!("puppet-{i} dropped: {e:?}"),
        }
    });
    let results = future::join_all(tasks).await;
    let ok = results.iter().filter(|r| r.is_ok()).count();
    for r in results.iter().filter_map(|r| r.as_ref().err()) { println!("ERR {r:#}"); }
    println!("{ok}/{n} puppets connected and held 15s");
    Ok(())
}
