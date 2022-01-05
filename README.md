# freeswitch-esl (WIP)

FreeSwitch ESL implementation for Rust

# Examples

## Inbound Example

```rust
use freeswitch_esl::{inbound::Inbound, EslError};

#[tokio::main]
async fn main() -> Result<(), EslError> {
    let addr = "3.109.206.34:8021"; // Freeswitch host
    let password = "ClueCon";
    let inbound = Inbound::new(addr, password).await?;

    let reloadxml = inbound.api("reloadxml").await?;
    println!("reloadxml response : {:?}", reloadxml);

    let reloadxml = inbound.bgapi("reloadxml").await?;
    println!("reloadxml response : {:?}", reloadxml);

    Ok(())
}
```

## Outbound Example

```rust
use freeswitch_esl::{
    outbound::{Outbound, OutboundSession},
    EslError,
};

async fn process_call(conn: OutboundSession) -> Result<(), EslError> {
    conn.answer().await?;
    conn.playback("ivr/ivr-welcome.wav").await?;
    conn.playback("misc/misc-freeswitch_is_state_of_the_art.wav").await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), EslError> {
    env_logger::init();
    let addr = "0.0.0.0:8085"; // Listen ip
    let listener = Outbound::bind(addr).await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move { process_call(socket).await });
    }
}
```
