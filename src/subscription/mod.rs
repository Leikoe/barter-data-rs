use crate::exchange::StreamSelector;
use barter_integration::{
    error::SocketError,
    model::{Instrument, InstrumentKind, SubscriptionId, Symbol},
    protocol::websocket::WsMessage,
    Validator,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
};

/// OrderBook [`SubKind`]s and the associated Barter output data models.
pub mod book;

/// Candle [`SubKind`] and the associated Barter output data model.
pub mod candle;

/// Liquidation [`SubKind`] and the associated Barter output data model.
pub mod liquidation;

/// Public trade [`SubKind`] and the associated Barter output data model.
pub mod trade;

/// Defines the type of a [`Subscription`], and the output [`Self::Event`] that it yields.
pub trait SubKind
where
    Self: Debug + Clone,
{
    type Event: Debug;
}

/// Barter [`Subscription`] used to subscribe to a [`SubKind`] for a particular exchange
/// [`Instrument`].
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct Subscription<Exchange, Kind> {
    pub exchange: Exchange,
    #[serde(flatten)]
    pub instrument: Instrument,
    #[serde(alias = "type")]
    pub kind: Kind,
}

/// Barter time interval used for specifying the interval of a [`SubKind::Candle`].
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum Interval {
    #[serde(alias = "1m")]
    Minute1,
    #[serde(alias = "3m")]
    Minute3,
    #[serde(alias = "5m")]
    Minute5,
    #[serde(alias = "15m")]
    Minute15,
    #[serde(alias = "30m")]
    Minute30,
    #[serde(alias = "1h")]
    Hour1,
    #[serde(alias = "2h")]
    Hour2,
    #[serde(alias = "4h")]
    Hour4,
    #[serde(alias = "6h")]
    Hour6,
    #[serde(alias = "8h")]
    Hour8,
    #[serde(alias = "12h")]
    Hour12,
    #[serde(alias = "1d")]
    Day1,
    #[serde(alias = "3d")]
    Day3,
    #[serde(alias = "1w")]
    Week1,
    #[serde(alias = "1M")]
    Month1,
    #[serde(alias = "3M")]
    Month3,
}

impl Display for Interval {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Interval::Minute1 => "1m",
                Interval::Minute3 => "3m",
                Interval::Minute5 => "5m",
                Interval::Minute15 => "15m",
                Interval::Minute30 => "30m",
                Interval::Hour1 => "1h",
                Interval::Hour2 => "2h",
                Interval::Hour4 => "4h",
                Interval::Hour6 => "6h",
                Interval::Hour8 => "8h",
                Interval::Hour12 => "12h",
                Interval::Day1 => "1d",
                Interval::Day3 => "3d",
                Interval::Week1 => "1w",
                Interval::Month1 => "1M",
                Interval::Month3 => "3M",
            }
        )
    }
}

impl<Exchange, Kind> Display for Subscription<Exchange, Kind>
where
    Exchange: Display,
    Kind: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}{}", self.exchange, self.kind, self.instrument)
    }
}

impl<Exchange, S, Kind> From<(Exchange, S, S, InstrumentKind, Kind)>
    for Subscription<Exchange, Kind>
where
    S: Into<Symbol>,
{
    fn from(
        (exchange, base, quote, instrument_kind, kind): (Exchange, S, S, InstrumentKind, Kind),
    ) -> Self {
        Self::new(exchange, (base, quote, instrument_kind), kind)
    }
}

impl<Exchange, I, Kind> From<(Exchange, I, Kind)> for Subscription<Exchange, Kind>
where
    I: Into<Instrument>,
{
    fn from((exchange, instrument, kind): (Exchange, I, Kind)) -> Self {
        Self::new(exchange, instrument, kind)
    }
}

impl<Exchange, Kind> Subscription<Exchange, Kind> {
    /// Constructs a new [`Subscription`] using the provided configuration.
    pub fn new<I>(exchange: Exchange, instrument: I, kind: Kind) -> Self
    where
        I: Into<Instrument>,
    {
        Self {
            exchange,
            instrument: instrument.into(),
            kind,
        }
    }
}

impl<Exchange, Kind> Validator for &Subscription<Exchange, Kind>
where
    Exchange: StreamSelector<Kind>,
    Kind: SubKind,
{
    fn validate(self) -> Result<Self, SocketError>
    where
        Self: Sized,
    {
        // Determine ExchangeId associated with this Subscription
        let exchange = Exchange::ID;

        // Validate the Exchange supports the Subscription InstrumentKind
        match self.instrument.kind {
            InstrumentKind::Spot if exchange.supports_spot() => Ok(self),
            InstrumentKind::FuturePerpetual if exchange.supports_futures() => Ok(self),
            other => Err(SocketError::Unsupported {
                entity: exchange.as_str(),
                item: other.to_string(),
            }),
        }
    }
}

/// Metadata generated from a collection of Barter [`Subscription`]s, including the exchange
/// specific subscription payloads that are sent to the exchange.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SubscriptionMeta {
    /// `HashMap` containing the mapping between a [`SubscriptionId`] and
    /// it's associated Barter [`Instrument`].
    pub instrument_map: Map<Instrument>,
    /// Collection of [`WsMessage`]s containing exchange specific subscription payloads to be sent.
    pub subscriptions: Vec<WsMessage>,
}

/// New type`HashMap` that maps a [`SubscriptionId`] to some associated type `T`.
///
/// Used by [`ExchangeTransformer`](crate::transformer::ExchangeTransformer)s to identify the
/// Barter [`Instrument`] associated with incoming exchange messages.
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Map<T>(pub HashMap<SubscriptionId, T>);

impl<T> FromIterator<(SubscriptionId, T)> for Map<T> {
    fn from_iter<Iter>(iter: Iter) -> Self
    where
        Iter: IntoIterator<Item = (SubscriptionId, T)>,
    {
        Self(iter.into_iter().collect::<HashMap<SubscriptionId, T>>())
    }
}

impl<T> Map<T> {
    /// Find the `T` associated with the provided [`SubscriptionId`].
    pub fn find(&self, id: &SubscriptionId) -> Result<T, SocketError>
    where
        T: Clone,
    {
        self.0
            .get(id)
            .cloned()
            .ok_or_else(|| SocketError::Unidentifiable(id.clone()))
    }

    /// Find the mutable reference to `T` associated with the provided [`SubscriptionId`].
    pub fn find_mut(&mut self, id: &SubscriptionId) -> Result<&mut T, SocketError> {
        self.0
            .get_mut(id)
            .ok_or_else(|| SocketError::Unidentifiable(id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod subscription {
        use super::*;
        use crate::exchange::coinbase::Coinbase;
        use crate::exchange::okx::Okx;
        use crate::subscription::trade::PublicTrades;

        mod de {
            use super::*;
            use crate::exchange::binance::futures::BinanceFuturesUsd;
            use crate::exchange::binance::spot::BinanceSpot;
            use crate::exchange::gateio::futures::GateioFuturesUsd;
            use crate::exchange::okx::Okx;
            use crate::subscription::book::OrderBooksL2;
            use crate::subscription::trade::PublicTrades;

            #[test]
            fn test_subscription_okx_spot_public_trades() {
                let input = r#"
                {
                    "exchange": "okx",
                    "base": "btc",
                    "quote": "usdt",
                    "instrument_type": "spot",
                    "kind": "public_trades"
                }
                "#;

                serde_json::from_str::<Subscription<Okx, PublicTrades>>(input).unwrap();
            }

            #[test]
            fn test_subscription_binance_spot_public_trades() {
                let input = r#"
                {
                    "exchange": "binance_spot",
                    "base": "btc",
                    "quote": "usdt",
                    "instrument_type": "spot",
                    "kind": "public_trades"
                }
                "#;

                serde_json::from_str::<Subscription<BinanceSpot, PublicTrades>>(input).unwrap();
            }

            #[test]
            fn test_subscription_binance_futures_usd_order_books_l2() {
                let input = r#"
                {
                    "exchange": "binance_futures_usd",
                    "base": "btc",
                    "quote": "usdt",
                    "instrument_type": "future_perpetual",
                    "kind": "order_books_l2"
                }
                "#;

                serde_json::from_str::<Subscription<BinanceFuturesUsd, OrderBooksL2>>(input)
                    .unwrap();
            }

            #[test]
            fn subscription_gateio_futures_usd_public_trades() {
                let input = r#"
                {
                    "exchange": "gateio_futures_usd",
                    "base": "btc",
                    "quote": "usdt",
                    "instrument_type": "future_perpetual",
                    "kind": "public_trades"
                }
                "#;

                serde_json::from_str::<Subscription<GateioFuturesUsd, PublicTrades>>(input)
                    .unwrap();
            }
        }

        #[test]
        fn test_validate_bitfinex_public_trades() {
            struct TestCase {
                input: Subscription<Coinbase, PublicTrades>,
                expected: Result<Subscription<Coinbase, PublicTrades>, SocketError>,
            }

            let tests = vec![
                TestCase {
                    // TC0: Valid Coinbase Spot PublicTrades subscription
                    input: Subscription::from((
                        Coinbase,
                        "base",
                        "quote",
                        InstrumentKind::Spot,
                        PublicTrades,
                    )),
                    expected: Ok(Subscription::from((
                        Coinbase,
                        "base",
                        "quote",
                        InstrumentKind::Spot,
                        PublicTrades,
                    ))),
                },
                TestCase {
                    // TC1: Invalid Coinbase FuturePerpetual PublicTrades subscription
                    input: Subscription::from((
                        Coinbase,
                        "base",
                        "quote",
                        InstrumentKind::FuturePerpetual,
                        PublicTrades,
                    )),
                    expected: Err(SocketError::Unsupported {
                        entity: "",
                        item: "".to_string(),
                    }),
                },
            ];

            for (index, test) in tests.into_iter().enumerate() {
                let actual = test.input.validate();
                match (actual, &test.expected) {
                    (Ok(actual), Ok(expected)) => {
                        assert_eq!(actual, expected, "TC{} failed", index)
                    }
                    (Err(_), Err(_)) => {
                        // Test passed
                    }
                    (actual, expected) => {
                        // Test failed
                        panic!("TC{index} failed because actual != expected. \nActual: {actual:?}\nExpected: {expected:?}\n");
                    }
                }
            }
        }

        #[test]
        fn test_validate_okx_public_trades() {
            struct TestCase {
                input: Subscription<Okx, PublicTrades>,
                expected: Result<Subscription<Okx, PublicTrades>, SocketError>,
            }

            let tests = vec![
                TestCase {
                    // TC0: Valid Okx Spot PublicTrades subscription
                    input: Subscription::from((
                        Okx,
                        "base",
                        "quote",
                        InstrumentKind::Spot,
                        PublicTrades,
                    )),
                    expected: Ok(Subscription::from((
                        Okx,
                        "base",
                        "quote",
                        InstrumentKind::Spot,
                        PublicTrades,
                    ))),
                },
                TestCase {
                    // TC1: Valid Okx FuturePerpetual PublicTrades subscription
                    input: Subscription::from((
                        Okx,
                        "base",
                        "quote",
                        InstrumentKind::FuturePerpetual,
                        PublicTrades,
                    )),
                    expected: Ok(Subscription::from((
                        Okx,
                        "base",
                        "quote",
                        InstrumentKind::FuturePerpetual,
                        PublicTrades,
                    ))),
                },
            ];

            for (index, test) in tests.into_iter().enumerate() {
                let actual = test.input.validate();
                match (actual, &test.expected) {
                    (Ok(actual), Ok(expected)) => {
                        assert_eq!(actual, expected, "TC{} failed", index)
                    }
                    (Err(_), Err(_)) => {
                        // Test passed
                    }
                    (actual, expected) => {
                        // Test failed
                        panic!("TC{index} failed because actual != expected. \nActual: {actual:?}\nExpected: {expected:?}\n");
                    }
                }
            }
        }
    }

    mod instrument_map {
        use super::*;

        #[test]
        fn test_find_instrument() {
            // Initialise SubscriptionId-InstrumentId HashMap
            let ids = Map(HashMap::from_iter([(
                SubscriptionId::from("present"),
                Instrument::from(("base", "quote", InstrumentKind::Spot)),
            )]));

            struct TestCase {
                input: SubscriptionId,
                expected: Result<Instrument, SocketError>,
            }

            let cases = vec![
                TestCase {
                    // TC0: SubscriptionId (channel) is present in the HashMap
                    input: SubscriptionId::from("present"),
                    expected: Ok(Instrument::from(("base", "quote", InstrumentKind::Spot))),
                },
                TestCase {
                    // TC1: SubscriptionId (channel) is not present in the HashMap
                    input: SubscriptionId::from("not present"),
                    expected: Err(SocketError::Unidentifiable(SubscriptionId::from(
                        "not present",
                    ))),
                },
            ];

            for (index, test) in cases.into_iter().enumerate() {
                let actual = ids.find(&test.input);
                match (actual, test.expected) {
                    (Ok(actual), Ok(expected)) => {
                        assert_eq!(actual, expected, "TC{} failed", index)
                    }
                    (Err(_), Err(_)) => {
                        // Test passed
                    }
                    (actual, expected) => {
                        // Test failed
                        panic!("TC{index} failed because actual != expected. \nActual: {actual:?}\nExpected: {expected:?}\n");
                    }
                }
            }
        }
    }
}
