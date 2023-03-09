/*
RUST_BACKTRACE=1 RUST_LOG=trace cargo run -p async-ping-cli --bin async_ping_ping -- 127.0.0.1
Or
cargo install async-ping-cli
async_ping_ping 127.0.0.1
*/

use core::time::Duration;
use std::{env, net::IpAddr};

use async_ping::{
    icmp_packet::{Icmp, Icmpv4, Icmpv6},
    PingClient,
};
use icmp_client::Config as ClientConfig;
use tracing_subscriber::{fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ip = env::args()
        .nth(1)
        .ok_or("args ip missing")?
        .parse::<IpAddr>()
        .map_err(|err| format!("args ip invalid, err:{err}"))?;

    //
    tracing_subscriber::registry().with(fmt::layer()).init();

    //
    let client = PingClient::<icmp_client::impl_tokio::Client>::new(
        ClientConfig::new(),
        ClientConfig::with_ipv6(),
    )?;

    {
        let client = client.clone();
        tokio::spawn(async move {
            client.handle_v4_recv_from().await;
        });
    }

    {
        let client = client.clone();
        tokio::spawn(async move {
            client.handle_v6_recv_from().await;
        });
    }

    //
    let payload = vec![0; 32];
    let timeout_dur = Duration::from_secs(2);

    for i in 0..1000 {
        let ret = match client
            .ping(ip, None, Some(i), payload.clone(), timeout_dur)
            .await
        {
            Ok((icmp, dur)) => match icmp {
                Icmp::V4(Icmpv4::EchoReply(echo_reply)) => Ok((dur, echo_reply.sequence_number)),
                Icmp::V4(Icmpv4::Other(tp, _, _)) => Err(format!("{tp:?}")),
                Icmp::V6(Icmpv6::EchoReply(echo_reply)) => Ok((dur, echo_reply.sequence_number)),
                Icmp::V6(Icmpv6::Other(tp, _, _)) => Err(format!("{tp:?}")),
            },
            Err(err) => Err(err.to_string()),
        };

        match ret {
            Ok((dur, sequence_number)) => {
                println!("icmp_seq={sequence_number} time={}ms", dur.as_millis())
            }
            Err(err) => println!("err={err}"),
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}
