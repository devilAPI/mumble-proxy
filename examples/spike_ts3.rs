//! Spike: connect to a real TS3 server as a client, print channels and clients, stay connected.
use anyhow::Result;
use futures::prelude::*;
use tokio::time::{self, Duration};
use tsclientlib::{Connection, DisconnectOptions, StreamItem};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let addr = std::env::args().nth(1).unwrap_or_else(|| "localhost".into());
    let mut con = Connection::build(addr).name("mumble-proxy-spike").connect()?;

    // Wait until the initial book is populated.
    con.events()
        .try_filter(|e| future::ready(matches!(e, StreamItem::BookEvents(_))))
        .next()
        .await
        .transpose()?;

    // Let the remaining initial notifications (channellist etc.) arrive.
    {
        let mut ev = con.events();
        let _ = time::timeout(Duration::from_secs(3), async { while ev.next().await.is_some() {} }).await;
    }
    {
        let state = con.get_state()?;
        println!("server: {}", state.server.name);
        for ch in state.channels.values() {
            println!("channel {:?} parent={:?} name={}", ch.id, ch.parent, ch.name);
        }
        for cl in state.clients.values() {
            println!("client {:?} name={} channel={:?}", cl.id, cl.name, cl.channel);
        }
    }

    // Stay connected for a while to prove keep-alive works.
    let mut events = con.events().try_filter(|_| future::ready(false));
    tokio::select! {
        _ = time::sleep(Duration::from_secs(20)) => println!("still connected after 20s"),
        e = events.next() => anyhow::bail!("disconnected: {e:?}"),
    }
    drop(events);
    con.disconnect(DisconnectOptions::new())?;
    con.events().for_each(|_| future::ready(())).await;
    Ok(())
}
