// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Demo [Exonum][exonum] service implementing a simple cryptocurrency.
//! See [the documentation][docs] for a detailed step-by-step guide how to approach this demo,
//! and [the repository README][readme] on how to use, test, and contribute to it.
//!
//! **Note.** The service in this crate is intended for demo purposes only. It is not intended
//! for use in production.
//!
//! [exonum]: https://github.com/exonum/exonum
//! [docs]: https://exonum.com/doc/get-started/create-service
//! [readme]: https://github.com/exonum/cryptocurrency#readme

#![deny(missing_debug_implementations, unsafe_code, bare_trait_objects)]

#[macro_use]
extern crate exonum;
#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

/// Unique service ID.
const RATES_SERVICE_ID: u16 = 128;
/// Name of the service.
pub const RATES_SERVICE_NAME: &str = "rates";

/// Persistent data.
pub mod schema {
    use exonum::{
        crypto::{Hash}, storage::{Fork, MapIndex, Snapshot},
    };

    // Declare the data to be stored in the blockchain, namely wallets with balances.
    // See [serialization docs][1] for details.
    //
    // [1]: https://exonum.com/doc/architecture/serialization

    encoding_struct! {
        struct Rate {
            exchange_id: u64,
            exchange_name: &str,
            trade_symbol: &str,
            trade_asset: &str,
            trade_currency: &str,
            trade_type: &str,
            trade_is_fiat: u64,
            trade_exchange_id: &str,
            trade_exchange_ts: u64,
            trade_exchange_price: u64,
            trade_exchange_amount: u64,
            trade_exchange_total: u64,
            trade_exchange_direction: &str,
            addition_data: &str,
        }
    }

    #[derive(Debug)]
    pub struct RatesSchema<T> {
        view: T,
    }

    impl<T: AsRef<dyn Snapshot>> RatesSchema<T> {
        /// Creates a new schema instance.
        pub fn new(view: T) -> Self {
            RatesSchema { view }
        }

        /// Returns an immutable version of the rate table.
        pub fn rates(&self) -> MapIndex<&dyn Snapshot, Hash, Rate> {
            MapIndex::new("rates.rate", self.view.as_ref())
        }

        /// Gets a specific rate from the storage.
        pub fn rate(&self, tx_hash: &Hash) -> Option<Rate> {
            self.rates().get(tx_hash)
        }
    }

    impl<'a> RatesSchema<&'a mut Fork> {
        /// Returns a mutable version of the rate table.
        pub fn rates_mut(&mut self) -> MapIndex<&mut Fork, Hash, Rate> {
            MapIndex::new("rates.rate", &mut self.view)
        }
    }
}

/// Transactions.
pub mod transactions {
    use RATES_SERVICE_ID;

    transactions! {
        pub RatesTransactions {
            const SERVICE_ID = RATES_SERVICE_ID;

            struct TxCreateRate {
                exchange_id: u64,
                exchange_name: &str,
                trade_symbol: &str,
                trade_asset: &str,
                trade_currency: &str,
                trade_type: &str,
                trade_is_fiat: u64,
                trade_exchange_id: &str,
                trade_exchange_ts: u64,
                trade_exchange_price: u64,
                trade_exchange_amount: u64,
                trade_exchange_total: u64,
                trade_exchange_direction: &str,
                addition_data: &str,
            }
        }
    }
}

/// Contract errors.
pub mod errors {
    // Workaround for `failure` see https://github.com/rust-lang-nursery/failure/issues/223 and
    // ECR-1771 for the details.
    #![allow(bare_trait_objects)]

    use exonum::blockchain::ExecutionError;

    /// Error codes emitted by `TxCreateWallet` and/or `TxTransfer` transactions during execution.
    #[derive(Debug, Fail)]
    #[repr(u8)]
    pub enum Error {
        #[fail(display = "Rate already exists")]
        RateAlreadyExists = 0,
    }

    impl From<Error> for ExecutionError {
        fn from(value: Error) -> ExecutionError {
            let description = format!("{}", value);
            ExecutionError::with_description(value as u8, description)
        }
    }
}

/// Contracts.
pub mod contracts {
    use exonum::{
        blockchain::{ExecutionResult, Transaction}, storage::Fork,
        crypto::CryptoHash,
    };

    use errors::Error;
    use schema::{RatesSchema, Rate};
    use transactions::{TxCreateRate};

    impl Transaction for TxCreateRate {
        fn verify(&self) -> bool {
            println!("verify()");
            return true
        }

        fn execute(&self, view: &mut Fork) -> ExecutionResult {
            println!("execute()");
            let mut schema = RatesSchema::new(view);
            if schema.rate(&self.hash()).is_none() {
                let rate = Rate::new(self.exchange_id(),
                                     self.exchange_name(),
                                     self.trade_symbol(),
                                     self.trade_asset(),
                                     self.trade_currency(),
                                     self.trade_type(),
                                     self.trade_is_fiat(),
                                     self.trade_exchange_id(),
                                     self.trade_exchange_ts(),
                                     self.trade_exchange_price(),
                                     self.trade_exchange_amount(),
                                     self.trade_exchange_total(),
                                     self.trade_exchange_direction(),
                                     self.addition_data()
                                    );
                println!("Creating the rate: {:?}", rate);
                schema.rates_mut().put(&self.hash(), rate);
                Ok(())
            } else {
                println!("The rate is already exists");
                Err(Error::RateAlreadyExists)?
            }
        }
    }
}

/// REST API.
pub mod api {
    use exonum::{
        api::{self, ServiceApiBuilder, ServiceApiState}, blockchain::Transaction,
        crypto::Hash, node::TransactionSend,
    };

    use transactions::{RatesTransactions};

    /// The structure returned by the REST API.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct TransactionResponse {
        /// Hash of the transaction.
        pub tx_hash: Hash,
    }

    #[derive(Debug, Clone)]
    pub struct RatesApi;

    impl RatesApi {
        /// Common processing for transaction-accepting endpoints.
        pub fn process_transaction(
            state: &ServiceApiState,
            query: RatesTransactions,
        ) -> api::Result<TransactionResponse> {
            println!("post_transaction()");
            let transaction: Box<dyn Transaction> = query.into();
            let tx_hash = transaction.hash();
            state.sender().send(transaction)?;
            Ok(TransactionResponse { tx_hash })
        }

        /// 'ServiceApiBuilder' facilitates conversion between transactions/read requests and REST
        /// endpoints; for example, it parses `POST`ed JSON into the binary transaction
        /// representation used in Exonum internally.
        pub fn wire(builder: &mut ServiceApiBuilder) {
            // Binds handlers to specific routes.
            builder
                .public_scope()
                .endpoint_mut("v1/rate", Self::process_transaction);
        }
    }
}

/// Service declaration.
pub mod service {
    use exonum::{
        api::ServiceApiBuilder, blockchain::{Service, Transaction, TransactionSet}, crypto::Hash,
        encoding, messages::RawTransaction, storage::Snapshot,
    };

    use api::{RatesApi};
    use transactions::{RatesTransactions};
    use RATES_SERVICE_ID;
    use RATES_SERVICE_NAME;

    #[derive(Debug)]
    pub struct RatesService;

    impl Service for RatesService {
        fn service_name(&self) -> &'static str {
            RATES_SERVICE_NAME
        }

        fn service_id(&self) -> u16 {
            RATES_SERVICE_ID
        }

        fn tx_from_raw(
            &self,
            raw: RawTransaction,
        ) -> Result<Box<dyn Transaction>, encoding::Error> {
            println!("tx_from_raw()");
            let tx = RatesTransactions::tx_from_raw(raw)?;
            Ok(tx.into())
        }

        fn state_hash(&self, _: &dyn Snapshot) -> Vec<Hash> {
            vec![]
        }

        fn wire_api(&self, builder: &mut ServiceApiBuilder) {
            RatesApi::wire(builder);
        }
    }
}
