#![allow(non_snake_case)]
use discord_presence::{Client, Event};
use std::{thread, time};

mod app;
mod data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let id_var = std::env!("DISCORD_ID");
    let client_id = id_var
        .parse::<u64>()
        .expect("Couldn't parse DISCORD_ID as u64");

    let mut drpc = Client::new(client_id);

    drpc.on_ready(|_ctx| {
        println!("READY!");
    })
    .persist();

    drpc.on_error(|ctx| {
        eprintln!("An error occurred: {:?}", ctx.event);
    })
    .persist();

    let mut app = app::App::default();

    drpc.start();

    drpc.block_until_event(Event::Ready)?;

    assert!(Client::is_ready());

    app.set_client(drpc);

    loop {
        app.update().await;
        thread::sleep(time::Duration::from_secs(1));
    }

    #[allow(unreachable_code)]
    Ok(())
}
