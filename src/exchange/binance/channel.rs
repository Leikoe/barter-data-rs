use super::{futures::BinanceFuturesUsd, Binance};
use crate::subscription::Interval;
use crate::{
    subscription::{
        book::{OrderBooksL1, OrderBooksL2},
        candle::Candles,
        liquidation::Liquidations,
        trade::PublicTrades,
        Subscription,
    },
    Identifier,
};
use serde::Serialize;
use std::fmt::{Debug, Display};

/// Type that defines how to translate a Barter [`Subscription`] into a [`Binance`](super::Binance)
/// channel to be subscribed to.
///
/// See docs: <https://binance-docs.github.io/apidocs/spot/en/#websocket-market-streams>
/// See docs: <https://binance-docs.github.io/apidocs/futures/en/#websocket-market-streams>
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize)]
pub struct BinanceChannel(pub &'static str);

impl BinanceChannel {
    /// [`Binance`](super::Binance) real-time trades channel name.
    ///
    /// See docs: <https://binance-docs.github.io/apidocs/spot/en/#trade-streams>
    ///
    /// Note:
    /// For [`BinanceFuturesUsd`](super::futures::BinanceFuturesUsd) this real-time
    /// stream is undocumented.
    ///
    /// See discord: <https://discord.com/channels/910237311332151317/923160222711812126/975712874582388757>
    pub const TRADES: Self = Self("@trade");

    /// [`Binance`](super::Binance) real-time OrderBook Level1 (top of book) channel name.
    ///
    /// See docs:<https://binance-docs.github.io/apidocs/spot/en/#individual-symbol-book-ticker-streams>
    /// See docs:<https://binance-docs.github.io/apidocs/futures/en/#individual-symbol-book-ticker-streams>
    pub const ORDER_BOOK_L1: Self = Self("@bookTicker");

    /// [`Binance`](super::Binance) OrderBook Level2 channel name (100ms delta updates).
    ///
    /// See docs: <https://binance-docs.github.io/apidocs/spot/en/#diff-depth-stream>
    /// See docs: <https://binance-docs.github.io/apidocs/futures/en/#diff-book-depth-streams>
    pub const ORDER_BOOK_L2: Self = Self("@depth@100ms");

    /// [`BinanceFuturesUsd`](super::futures::BinanceFuturesUsd) liquidation orders channel name.
    ///
    /// See docs: <https://binance-docs.github.io/apidocs/futures/en/#liquidation-order-streams>
    pub const LIQUIDATIONS: Self = Self("@forceOrder");

    pub const CANDLES: Self = Self("@kline_");
}

impl<Server> Identifier<BinanceChannel> for Subscription<Binance<Server>, PublicTrades> {
    fn id(&self) -> BinanceChannel {
        BinanceChannel::TRADES
    }
}

impl<Server> Identifier<BinanceChannel> for Subscription<Binance<Server>, Candles> {
    fn id(&self) -> BinanceChannel {
        match self.kind.0 {
            Interval::Minute1 => BinanceChannel("@kline_1m"),
            Interval::Minute3 => BinanceChannel("@kline_3m"),
            Interval::Minute5 => BinanceChannel("@kline_5m"),
            Interval::Minute15 => BinanceChannel("@kline_15m"),
            Interval::Minute30 => BinanceChannel("@kline_30m"),
            Interval::Hour1 => BinanceChannel("@kline_1h"),
            Interval::Hour2 => BinanceChannel("@kline_2h"),
            Interval::Hour4 => BinanceChannel("@kline_4h"),
            Interval::Hour6 => BinanceChannel("@kline_6h"),
            Interval::Hour8 => BinanceChannel("@kline_8h"),
            Interval::Hour12 => BinanceChannel("@kline_12h"),
            Interval::Day1 => BinanceChannel("@kline_1d"),
            Interval::Day3 => BinanceChannel("@kline_3d"),
            Interval::Week1 => BinanceChannel("@kline_1w"),
            Interval::Month1 => BinanceChannel("@kline_1M"),
            Interval::Month3 => BinanceChannel("@kline_3M"),
        }
    }
}

impl<Server> Identifier<BinanceChannel> for Subscription<Binance<Server>, OrderBooksL1> {
    fn id(&self) -> BinanceChannel {
        BinanceChannel::ORDER_BOOK_L1
    }
}

impl<Server> Identifier<BinanceChannel> for Subscription<Binance<Server>, OrderBooksL2> {
    fn id(&self) -> BinanceChannel {
        BinanceChannel::ORDER_BOOK_L2
    }
}

impl Identifier<BinanceChannel> for Subscription<BinanceFuturesUsd, Liquidations> {
    fn id(&self) -> BinanceChannel {
        BinanceChannel::LIQUIDATIONS
    }
}

impl AsRef<str> for BinanceChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}
