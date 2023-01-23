use crate::{
    event::{MarketEvent, MarketIter},
    exchange::{ExchangeId, ExchangeSub},
    subscription::trade::PublicTrade,
    Identifier,
};
use barter_integration::model::{Exchange, Instrument, SubscriptionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::exchange::binance::channel::BinanceChannel;
use crate::subscription::candle::Candle;

/// Binance real-time trade message.
///
/// Note:
/// For [`BinanceFuturesUsd`](super::futures::BinanceFuturesUsd) this real-time stream is
/// undocumented.
///
/// See discord: <https://discord.com/channels/910237311332151317/923160222711812126/975712874582388757>
///
/// ### Raw Payload Examples
/// See docs: <https://binance-docs.github.io/apidocs/spot/en/#trade-streams>
/// #### Spot Side::Buy Trade
/// ```json
/// {
///     "e":"trade",
///     "E":1649324825173,
///     "s":"ETHUSDT",
///     "t":1000000000,
///     "p":"10000.19",
///     "q":"0.239000",
///     "b":10108767791,
///     "a":10108764858,
///     "T":1749354825200,
///     "m":false,
///     "M":true
/// }
/// ```
///
/// #### FuturePerpetual Side::Sell Trade
/// ```json
/// {
///     "e": "trade",
///     "E": 1649839266194,
///     "T": 1749354825200,
///     "s": "ETHUSDT",
///     "t": 1000000000,
///     "p":"10000.19",
///     "q":"0.239000",
///     "X": "MARKET",
///     "m": true
/// }
/// ```
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BinanceCandle {
    #[serde(alias = "s", deserialize_with = "de_trade_subscription_id")]
    pub subscription_id: SubscriptionId,
    #[serde(
        alias = "E",
        deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc"
    )]
    pub time: DateTime<Utc>,
    #[serde(alias = "k")]
    pub candle: BinanceCandleInner,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BinanceCandleInner {
    #[serde(
        alias = "t",
        deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc"
    )]
    pub start: DateTime<Utc>,
    #[serde(
        alias = "T",
        deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc"
    )]
    pub end: DateTime<Utc>,
    #[serde(alias = "i")]
    pub interval: String,
    #[serde(alias = "o", deserialize_with = "barter_integration::de::de_str")]
    pub open: f64,
    #[serde(alias = "c", deserialize_with = "barter_integration::de::de_str")]
    pub close: f64,
    #[serde(alias = "h", deserialize_with = "barter_integration::de::de_str")]
    pub high: f64,
    #[serde(alias = "l", deserialize_with = "barter_integration::de::de_str")]
    pub low: f64,
    #[serde(alias = "v", deserialize_with = "barter_integration::de::de_str")]
    pub volume: f64,
    #[serde(alias = "n")]
    pub trades: u64,
}

impl Identifier<Option<SubscriptionId>> for BinanceCandle {
    fn id(&self) -> Option<SubscriptionId> {
        Some(self.subscription_id.clone())
    }
}

impl From<(ExchangeId, Instrument, BinanceCandle)> for MarketIter<Candle> {
    fn from((exchange_id, instrument, trade): (ExchangeId, Instrument, BinanceCandle)) -> Self {
        Self(vec![Ok(MarketEvent {
            exchange_time: trade.time,
            received_time: Utc::now(),
            exchange: Exchange::from(exchange_id),
            instrument,
            kind: Candle {
                close_time: trade.candle.end,
                open: trade.candle.open,
                high: trade.candle.high,
                low: trade.candle.low,
                close: trade.candle.close,
                volume: trade.candle.volume,
                trade_count: trade.candle.trades
            },
        })])
    }
}

/// Deserialize a [`BinanceCandle`] "s" (eg/ "BTCUSDT") as the associated [`SubscriptionId`]
/// (eg/ "@trade|BTCUSDT").
pub fn de_trade_subscription_id<'de, D>(deserializer: D) -> Result<SubscriptionId, D::Error>
    where
        D: serde::de::Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer)
        .map(|market| ExchangeSub::from((BinanceChannel::CANDLES, market)).id())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    mod de {
        use super::*;
        use barter_integration::de::datetime_utc_from_epoch_duration;
        use barter_integration::error::SocketError;
        use serde::de::Error;
        use std::time::Duration;
        
        #[test]
        fn test_binance_trade() {
            struct TestCase {
                input: &'static str,
                expected: Result<BinanceCandle, SocketError>,
            }
            
            let tests = vec![
                TestCase {
                    // TC0: Spot trade valid
                    input: r#"
                    {
                      "e": "kline",
                      "E": 123456789,
                      "s": "BNBBTC",
                      "k": {
                        "t": 123400000,
                        "T": 123460000,
                        "s": "BNBBTC",
                        "i": "1m",
                        "f": 100,
                        "L": 200,
                        "o": "0.0010",
                        "c": "0.0020",
                        "h": "0.0025",
                        "l": "0.0015",
                        "v": "1000",
                        "n": 100,
                        "x": false,
                        "q": "1.0000",
                        "V": "500",
                        "Q": "0.500",
                        "B": "123456"
                      }
                    }
                    "#,
                    expected: Ok(BinanceCandle {
                        subscription_id: SubscriptionId::from("@kline_1m|BNBBTC"),
                        time: datetime_utc_from_epoch_duration(Duration::from_millis(
                            123456789,
                        )),
    
                        candle: BinanceCandleInner {
                            start: datetime_utc_from_epoch_duration(Duration::from_millis(
                                123400000,
                            )),
                            end: datetime_utc_from_epoch_duration(Duration::from_millis(
                                123460000,
                            )),
                            interval: "1m".to_string(),
                            open: 0.0010,
                            close: 0.0020,
                            high: 0.0025,
                            low: 0.0015,
                            volume: 1000.0,
                            trades: 100
                        },
                    }),
                },
            ];
            
            for (index, test) in tests.into_iter().enumerate() {
                let actual = serde_json::from_str::<BinanceCandle>(test.input);
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
