//! Spike: connect to an unmodified murmur with a generated client certificate,
//! print channel tree and users after ServerSync, keep alive with pings for 20s.
use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use mumble_protocol_2x::control::{msgs, ClientControlCodec, ControlPacket};
use tokio::net::TcpStream;
use tokio::time::{self, Duration, Instant};
use tokio_util::codec::Decoder;

#[tokio::main]
async fn main() -> Result<()> {
    let host = std::env::args().nth(1).unwrap_or_else(|| "localhost".into());
    let name = std::env::args().nth(2).unwrap_or_else(|| "ts3-spike".into());

    // Self-signed client cert: Mumble identifies (registered) users by its hash.
    let key = rcgen::KeyPair::generate()?;
    let cert = rcgen::CertificateParams::new(vec![name.clone()])?.self_signed(&key)?;
    let identity = native_tls::Identity::from_pkcs8(cert.pem().as_bytes(), key.serialize_pem().as_bytes())?;
    let connector = native_tls::TlsConnector::builder()
        .identity(identity)
        .danger_accept_invalid_certs(true) // murmur ships a self-signed cert; spike only
        .build()?;
    let tcp = TcpStream::connect((host.as_str(), 64738)).await?;
    let tls = tokio_native_tls::TlsConnector::from(connector).connect(&host, tcp).await?;
    let (mut sink, mut stream) = ClientControlCodec::new().framed(tls).split();

    let mut v = msgs::Version::new();
    v.set_version_v1((1 << 16) | (4 << 8)); // announce 1.4 -> legacy UDP voice format
    v.set_release("mumble-proxy spike".into());
    sink.send(v.into()).await?;
    let mut a = msgs::Authenticate::new();
    a.set_username(name);
    a.set_opus(true);
    sink.send(a.into()).await?;

    let mut channels = std::collections::BTreeMap::new();
    let mut users = std::collections::BTreeMap::new();
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut ping = time::interval(Duration::from_secs(5));
    let mut synced = false;
    loop {
        tokio::select! {
            _ = time::sleep_until(deadline) => break,
            _ = ping.tick() => sink.send(msgs::Ping::new().into()).await?,
            p = stream.next() => match p.context("server closed")?? {
                ControlPacket::ChannelState(c) => { channels.insert(c.channel_id(), (c.parent(), c.name().to_string(), c.temporary())); }
                ControlPacket::UserState(u) => { users.insert(u.session(), (u.name().to_string(), u.channel_id())); }
                ControlPacket::Reject(r) => anyhow::bail!("rejected: {r:?}"),
                ControlPacket::ServerSync(s) => {
                    synced = true;
                    println!("ServerSync: own session {} welcome={:?}", s.session(), s.welcome_text());
                    for (id, (parent, n, temp)) in &channels { println!("channel {id} parent={parent} temp={temp} name={n}"); }
                    for (sid, (n, ch)) in &users { println!("user {sid} name={n} channel={ch}"); }
                }
                _ => {}
            }
        }
    }
    anyhow::ensure!(synced, "never got ServerSync");
    println!("still connected after 20s");
    Ok(())
}
