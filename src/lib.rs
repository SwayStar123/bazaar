#![deny(unused_must_use)]
#![deny(unsafe_code)]
#![allow(clippy::new_without_default)]

pub mod apis;
mod asset;
mod candle;
mod exchange;
mod market;
pub mod strategies;
mod wallet;

pub use asset::*;
pub use candle::*;
use chrono::{DateTime, Duration, TimeZone, Utc};
pub use exchange::*;
pub use market::*;
use rust_decimal_macros::dec;
pub use wallet::*;

use apis::{Api, ForwardFill, Simulate, Store};
use rust_decimal::Decimal;
use strategies::{Monitor, Strategy};

pub struct Bazaar {
    pub start_capital: Decimal,
    pub start_time: DateTime<Utc>,
}

impl Default for Bazaar {
    fn default() -> Self {
        Bazaar {
            start_capital: dec!(1000),
            start_time: Utc.ymd(2021, 1, 1).and_hms(0, 0, 0),
        }
    }
}

impl Bazaar {
    /// Runs your strategy live.
    #[cfg(not(feature = "backtest"))]
    pub async fn run<A: Api, B: Api, S: Strategy<B>>(
        self,
        api: A,
        strategy: S,
    ) -> Result<(), AnyError>
    where
        Monitor<B, S>: Strategy<Simulate<ForwardFill<Store<A>>>>,
    {
        let strategy = Monitor::new(strategy);
        let exchange = Exchange::new(api, self.start_time);
        exchange.run(strategy).await?;

        Ok(())
    }

    /// Runs your strategy in backtest mode.
    #[cfg(feature = "backtest")]
    pub async fn run<A: Api, B: Api, S: Strategy<B>>(
        self,
        api: A,
        strategy: S,
    ) -> Result<(), AnyError>
    where
        Monitor<B, S>: Strategy<Simulate<ForwardFill<Store<A>>>>,
    {
        let mut wallet = Wallet::new();
        wallet.deposit(self.start_capital, Asset::new("USD"));

        let strategy = Monitor::new(strategy);
        let api = Simulate::new(
            ForwardFill::new(Store::new(api).await, Duration::hours(24)),
            wallet,
        );
        let exchange = Exchange::new(api, self.start_time);
        exchange.run(strategy).await?;

        Ok(())
    }
}
