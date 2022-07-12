use bincode;
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use hex::{FromHex, FromHexError, ToHex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::{
    fmt::Debug,
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Config {
    pub genesis_file: Option<String>,
}

impl Config {
    pub fn new(mut args: impl Iterator<Item = String>) -> Result<Self, &'static str> {
        args.next();

        let genesis_file = args.next();

        Ok(Config { genesis_file })
    }
}

fn parse_hex_string<T>(string: &str) -> Result<T, T::Error>
where
    T: FromHex,
{
    <T>::from_hex(string)
}

pub fn parse_hex_string_as_bytes(hex_string: &str) -> Result<Bytes32, FromHexError> {
    let hex_string = hex_string.strip_prefix("0x").unwrap_or_else(|| hex_string);
    let bytes = parse_hex_string(hex_string).unwrap();

    Ok(Bytes(bytes))
}

pub fn parse_hex_string_as_address(hex_string: &str) -> Result<Address, FromHexError> {
    let hex_string = hex_string.strip_prefix("0x").unwrap_or_else(|| hex_string);
    let bytes = parse_hex_string(hex_string).unwrap();

    Ok(Bytes(bytes))
}

pub fn init_genesis(filename: &str) -> Result<Block, &'static str> {
    let json = fs::read_to_string(filename).unwrap();
    let genesis: Value = serde_json::from_str(&json).unwrap();

    let mut transactions: Vec<Transaction> = Vec::new();

    for tx in genesis["transactions"].as_array().unwrap().iter() {
        match (
            parse_hex_string_as_address(tx["from"].as_str().unwrap()),
            parse_hex_string_as_address(tx["to"].as_str().unwrap()),
            parse_hex_string_as_bytes(tx["hash"].as_str().unwrap()),
        ) {
            (Ok(from), Ok(to), Ok(hash)) => transactions.push(Transaction {
                from,
                to,
                value: tx["value"].as_u64().unwrap() as u32,
                hash,
            }),
            _ => return Err("Unable to process genesis file"),
        };
    }

    let mut genesis = Block {
        hash: Default::default(),
        parent_hash: Default::default(),
        transactions,
        timestamp: genesis["timestamp"].as_u64().unwrap(),
    };

    genesis.hash = genesis.hash_block(0);

    Ok(genesis)
}

pub fn run(config: Config) {
    let genesis = match config.genesis_file {
        Some(filename) => init_genesis(&filename).unwrap(),
        None => Block {
            hash: Default::default(),
            parent_hash: Default::default(),
            transactions: Default::default(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        },
    };

    let mut blockchain = Blockchain::new(genesis);

    loop {
        let block = blockchain.add_new_block();

        println!("⛏ New block mined!");
        println!(
            "Block hash: {} | Timestamp: {}",
            block.hash, block.timestamp
        );
    }
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new(genesis: Block) -> Self {
        Blockchain {
            chain: vec![genesis],
            transactions: Default::default(),
        }
    }

    pub fn add_pending_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn mine(&self, block: &mut Block) -> Bytes32 {
        let mut nonce: i64 = 0;

        block.hash = loop {
            let hash = block.hash_block(nonce);

            if hash.as_ref().starts_with(&[0, 0]) {
                break hash;
            }

            nonce += 1;
        };

        block.hash
    }

    pub fn create_block(&self) -> Block {
        let latest = &self.chain[self.chain.len() - 1];
        let mut new = Block::new(latest.hash);

        let mut tx_iter = self.transactions.iter();

        for _ in 1..2 {
            if let Some(tx) = tx_iter.next() {
                new.transactions.push(tx.clone());
            }
        }

        new
    }

    pub fn add_new_block(&mut self) -> Block {
        let mut block = self.create_block();
        self.mine(&mut block);
        self.chain.push(block.clone());
        block
    }
}

#[derive(Clone, PartialEq, Default, Copy, Deserialize, Serialize)]
pub struct Bytes<T: AsRef<[u8]>>(T);

type Bytes32 = Bytes<[u8; 32]>;

type Address = Bytes<[u8; 20]>;

macro_rules! impl_traits_for_bytes {
    (for $($t:ty), +) => {
        $(impl Debug for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }

        impl Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "0x{}", self.encode_hex::<String>())
            }
        }

        impl AsRef<[u8]> for $t {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        })*
    };
}

impl_traits_for_bytes!(for Bytes32, Address);

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Block {
    pub hash: Bytes32,
    pub parent_hash: Bytes32,
    pub transactions: Vec<Transaction>,
    pub timestamp: u64,
}

impl Block {
    pub fn new(parent_hash: Bytes32) -> Self {
        Block {
            hash: Default::default(),
            parent_hash,
            transactions: Default::default(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn set_transactions(&mut self, transactions: Vec<Transaction>) {
        self.transactions = transactions;
    }

    pub fn hash_block(&self, nonce: i64) -> Bytes32 {
        let encoded = bincode::serialize(&self).unwrap();

        let mut hash = [0u8; 32];
        let mut hasher = Sha3::keccak256();

        hasher.input(&encoded);
        hasher.input(&nonce.to_be_bytes());
        hasher.result(&mut hash);

        Bytes(hash)
    }
}

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub value: u32,
    pub hash: Bytes32,
}

// mod tests {
//     use super::*;
//     use std::clone;

// const GENESIS: Block = Block {
//     hash: [0; 32],
//     parent_hash: [0; 32],
//     transactions: Vec::new(),
// };

// const TRANSACTIONS: [Transaction; 2] = [Transaction {}, Transaction {}];

// fn init() -> Blockchain {
//     let mut blockchain = Blockchain::new(GENESIS);
//     blockchain.transactions = Vec::from(TRANSACTIONS);

//     blockchain
// }

// #[test]
// fn create_blockchain() {
//     let blockchain = init();

//     assert_eq!(blockchain.chain[0], GENESIS);
//     assert_eq!(blockchain.transactions[0..1], TRANSACTIONS);
// }

// #[test]
// fn create_block() {
//     let block = Block {
//         hash: ,
//         parent_hash: GENESIS.hash,
//         transactions: Vec
//     };

//     let blockchain = init();
//     let parent_hash = blockchain.chain[0].hash;

//     let mut txs: Vec<Transaction>;
//     txs.clone_from_slice(&blockchain.transactions[0..1]);

//     let mut new_block = Block::new(parent_hash, txs);

//     assert_eq!(block, new_block);
//     assert_eq!(block, new_block);
// }

// #[test]
// fn test_parse_hex_string() {
//     let hex_string = "9b00b72c6bb7a8761a0730b7f5d6090229aacbb57f96d83b4e5bc6866b385d32";
//     let test_bytes = [
//         155, 0, 183, 44, 107, 183, 168, 118, 26, 7, 48, 183, 245, 214, 9, 2, 41, 170, 203, 181,
//         127, 150, 216, 59, 78, 91, 198, 134, 107, 56, 93, 50,
//     ];
//     let bytes = parse_hex_string_as_bytes(hex_string);

//     // println!("{:?}", bytes);

//     assert_eq!(bytes, Bytes32(test_bytes));
// }
// }
