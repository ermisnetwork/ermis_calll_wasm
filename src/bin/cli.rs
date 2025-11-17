use std::f32::consts::E;

use anyhow::Result;
use ermis_call_node_wasm::node::ErmisCallEndpoint;
use clap::Parser;
use iroh::EndpointAddr;
use n0_future::StreamExt;
use base64::{Engine as _, prelude::BASE64_STANDARD};

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Connect {
        endpoint_addr: String,
    },
    Accept,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let mut node = ErmisCallEndpoint::new(&["https://test-iroh.ermis.network.:8443"], None).await?;
    let addr = node.get_local_endpoint_addr()?;
    println!("Node Endpoint Addr: {}", addr);
    match args.command {
        Command::Connect {
            endpoint_addr,
        } => {
            // let addr_bytes = BASE64_STANDARD.decode(&endpoint_addr)?;
            // let addr: EndpointAddr = bitcode::deserialize(&addr_bytes)?;
            let _ = node.connect(&endpoint_addr).await?;
            // let  _ = node.open_bidi_stream().await?;
            // node.async_send(payload.as_bytes()).await?;
            
        }
        Command::Accept => {

            println!("connect to this node:");
            println!(
                "cargo run -- connect {}",
                node.get_local_endpoint_addr()?,
            );
            let _ = node.accept_connection().await?;
            let _ = node.accept_bidi_stream().await?;

            loop {
        if let Ok(bytes) = node.recv() {
            println!(
                "Received bytes: {}, {:?}",
                u64::from_be_bytes(bytes[0..8].try_into().unwrap()),
                std::time::Instant::now(),
            );
        }
    }
        }
    }

    Ok(())
}
