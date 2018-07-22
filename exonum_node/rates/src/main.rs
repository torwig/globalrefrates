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

extern crate exonum;
extern crate rates;
#[macro_use] extern crate serde_derive;
extern crate toml;

use exonum::blockchain::{GenesisConfig, ConsensusConfig, ValidatorKeys};
use exonum::node::{Node, NodeApiConfig, NodeConfig, MemoryPoolConfig, ConnectInfo, ConnectListConfig};
use exonum::storage::{RocksDB, DbOptions};
use exonum::crypto::{PublicKey, SecretKey};
use exonum::events::{NetworkConfiguration};

use rates::service::{RatesService};

use std::fs::File;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct TNodeConfig {
    database_path: String,
    external_address: Option<SocketAddr>,
    listen_address: SocketAddr,
    peers: Vec<SocketAddr>,
    consensus_public_key: PublicKey,
    consensus_secret_key: SecretKey,
    service_public_key: PublicKey,
    service_secret_key: SecretKey,
    network: TNetworkConfig,
    api: TApiConfig,
    mempool: TMempoolConfig,
    genesis: TGenesisConfig,
    consensus: TConsensusConfig,
    connect_list: TConnectList,
}

#[derive(Deserialize, Debug)]
struct TNetworkConfig {
    max_incoming_connections: usize,
    max_outgoing_connections: usize,
    tcp_nodelay: bool,
    tcp_connect_retry_timeout: u64,
    tcp_connect_max_retries: u64
}

#[derive(Deserialize, Debug)]
struct TApiConfig {
    enable_blockchain_explorer: bool,
    state_update_timeout: usize,
    public_api_address: Option<SocketAddr>,
    private_api_address: Option<SocketAddr>
}

#[derive(Deserialize, Debug)]
struct TMempoolConfig {
    tx_pool_capacity: usize,
}

#[derive(Deserialize, Debug)]
struct TGenesisConfig {
    validator_keys: Vec<ValidatorKeys>,
}

#[derive(Deserialize, Debug)]
struct TConsensusConfig {
    max_message_len: u32,
    max_propose_timeout: u64,
    min_propose_timeout: u64,
    peers_timeout: u64,
    propose_timeout_threshold: u32,
    round_timeout: u64,
    status_timeout: u64,
    txs_block_limit: u32
}

#[derive(Deserialize, Debug)]
struct TConnectList {
    peers: Vec<ConnectInfo>
}

fn main() {
    exonum::helpers::init_logger().unwrap();
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];
    println!("Config file is: {}", filename);

    let mut f = File::open(filename).expect("file not found");
    let mut content = String::new();
    f.read_to_string(&mut content)
        .expect("something went wrong reading the file");
    let node_conf: TNodeConfig = toml::from_str(&content).unwrap();
    println!("TNodeConfig: {:?}", node_conf);

    let db_path = node_conf.database_path.clone();
    println!("Database path: {}", db_path);
    let db_path = Path::new(&db_path);
    let options = DbOptions::default();
    let database = match RocksDB::open(db_path, &options) {
        Ok(db) => {db},
        Err(e) => { panic!("Failed to open database: {:?}", e); }
    };

    let consensus_config = ConsensusConfig {
        round_timeout: node_conf.consensus.round_timeout,
        status_timeout: node_conf.consensus.status_timeout,
        peers_timeout: node_conf.consensus.peers_timeout,
        txs_block_limit: node_conf.consensus.txs_block_limit,
        max_message_len: node_conf.consensus.max_message_len,
        min_propose_timeout: node_conf.consensus.min_propose_timeout,
        max_propose_timeout: node_conf.consensus.max_propose_timeout,
        propose_timeout_threshold: node_conf.consensus.propose_timeout_threshold
    };

    let genesis_conf = GenesisConfig {
        consensus: consensus_config,
        validator_keys: node_conf.genesis.validator_keys
    };

    let api_conf = NodeApiConfig {
        state_update_timeout: node_conf.api.state_update_timeout,
        enable_blockchain_explorer: node_conf.api.enable_blockchain_explorer,
        public_api_address: node_conf.api.public_api_address,
        private_api_address: node_conf.api.private_api_address,
        ..Default::default()
    };

    let network_conf = NetworkConfiguration {
        max_incoming_connections: node_conf.network.max_incoming_connections,
        max_outgoing_connections: node_conf.network.max_outgoing_connections,
        tcp_nodelay: node_conf.network.tcp_nodelay,
        tcp_keep_alive: None,
        tcp_connect_retry_timeout: node_conf.network.tcp_connect_retry_timeout,
        tcp_connect_max_retries: node_conf.network.tcp_connect_max_retries
    };

    let mempool_conf = MemoryPoolConfig {
        tx_pool_capacity: node_conf.mempool.tx_pool_capacity,
        ..Default::default()
    };

    let conn_list_conf = ConnectListConfig {
        peers: node_conf.connect_list.peers
    };

    let config = NodeConfig {
        genesis: genesis_conf,
        listen_address: node_conf.listen_address,
        external_address: node_conf.external_address,
        network: network_conf,
        consensus_public_key: node_conf.consensus_public_key,
        consensus_secret_key: node_conf.consensus_secret_key,
        service_public_key: node_conf.service_public_key,
        service_secret_key: node_conf.service_secret_key,
        api: api_conf,
        mempool: mempool_conf,
        services_configs: Default::default(),
        database: Default::default(),
        connect_list: conn_list_conf,
    };

    let node = Node::new(
        database,
        vec![Box::new(RatesService)],
        config,
        Some(node_conf.database_path),
    );
    println!("Starting a single node...");
    println!("Blockchain is ready for transactions!");
    node.run().unwrap();
}
