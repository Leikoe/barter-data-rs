use barter_data::subscription::Interval;
use barter_data::{
    exchange::{binance::spot::BinanceSpot, ExchangeId},
    streams::Streams,
    subscription::candle::Candles,
};
use barter_integration::model::InstrumentKind;
use tracing::info;

#[rustfmt::skip]
#[tokio::main]
async fn main() {
    // Initialise INFO Tracing log subscriber
    init_logging();

    // Initialise PublicTrades Streams for BinanceFuturesUsd only
    // '--> each call to StreamBuilder::subscribe() creates a separate WebSocket connection
    let mut streams = Streams::<Candles>::builder()
        // Separate WebSocket connection for BTC_USDT stream since it's very high volume
        .subscribe([
            (BinanceSpot::default(), "btc", "usdt", InstrumentKind::Spot, Candles(Interval::Minute1)),
        ])
        .init()
        .await
        .unwrap();

    // Select the ExchangeId::BinanceFuturesUsd stream
    // Notes:
    //  - Use `streams.select(ExchangeId)` to interact with the individual exchange streams!
    //  - Use `streams.join()` to join all exchange streams into a single mpsc::UnboundedReceiver!
    let mut binance_stream = streams
        .select(ExchangeId::BinanceSpot)
        .unwrap();

    while let Some(trade) = binance_stream.recv().await {
        info!("MarketEvent<PublicTrade>: {trade:?}");
    }
}

// Initialise an INFO `Subscriber` for `Tracing` Json logs and install it as the global default.
fn init_logging() {
    tracing_subscriber::fmt()
        // Filter messages based on the INFO
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // Disable colours on release builds
        .with_ansi(cfg!(debug_assertions))
        // Enable Json formatting
        .json()
        // Install this Tracing subscriber as global default
        .init()
}
