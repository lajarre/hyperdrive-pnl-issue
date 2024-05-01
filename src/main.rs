use std::env;
use std::sync::Arc;

use ethers::{
    core::types::H160,
    providers::{Middleware, Provider, Ws},
};
use eyre::Result;
use hex_literal::hex;
use tracing::info;

use hyperdrive_math::State;
use hyperdrive_wrappers::wrappers::ihyperdrive::i_hyperdrive;

// First OpenLong of a trader: https://sepolia.etherscan.io/tx/0x9a06d8ebc7f35429a1bc7fba0ddd2b510bbb02d8477f3bff53a95ec8ab2891d6#eventlog
const EVENT_BLOCK_NUM: u64 = 5668890;

const CURRENT_BLOCK_NUM: u64 = 5669072;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let ws_url = env::var("WS_URL")?;
    let provider = Provider::<Ws>::connect(ws_url).await?;
    let client = Arc::new(provider);

    let contract = i_hyperdrive::IHyperdrive::new(
        H160(hex!("392839da0dacac790bd825c81ce2c5e264d793a8")),
        client.clone(),
    );

    let events = contract
        .event::<i_hyperdrive::OpenLongFilter>()
        .from_block(EVENT_BLOCK_NUM)
        .to_block(EVENT_BLOCK_NUM);
    let query = events.query().await?;

    for evt in query {
        info!(evt=?evt, "OpenLong");

        let current_time = client
            .get_block(CURRENT_BLOCK_NUM)
            .await?
            .unwrap()
            .timestamp;

        assert!(current_time < evt.maturity_time);

        let pool_config = contract
            .get_pool_config()
            .block(CURRENT_BLOCK_NUM)
            .call()
            .await?;
        let pool_info = contract
            .get_pool_info()
            .block(CURRENT_BLOCK_NUM)
            .call()
            .await?;
        let state = State::new(pool_config, pool_info);

        info!(bond_amount=?evt.bond_amount,
            maturity_time=?evt.maturity_time,
            current_time=?current_time, "CalculatingCloseLong");

        let close_long =
            state.calculate_close_long(evt.bond_amount, evt.maturity_time, current_time);

        info!(close_long=?close_long, "CloseLong");
    }

    Ok(())
}
