use bincode;
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use hex::ToHex;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::{
    fmt::Debug,
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn run() {
    let genesis = Block {
        hash: Default::default(),
        parent_hash: Default::default(),
        transactions: Default::default(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
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

#[derive(Clone, PartialEq, Default, Copy)]
pub struct Bytes32([u8; 32]);

impl Debug for Bytes32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Display for Bytes32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", self.encode_hex::<String>())
    }
}

impl AsRef<[u8]> for Bytes32 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, PartialEq, Clone)]
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

        Bytes32(hash)
    }
}

impl Serialize for Block {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("block", 3).unwrap();
        s.serialize_field("hash", &self.hash.as_ref())?;
        s.serialize_field("parent_hash", &self.parent_hash.as_ref())?;
        s.serialize_field("transactions", &self.transactions)?;
        s.serialize_field("timestamp", &self.timestamp)?;
        s.end()
    }
}

type Address = [u8; 20];

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub value: u32,
    pub hash: [u8; 32],
}

impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("transaction", 4).unwrap();
        s.serialize_field("from", &self.from)?;
        s.serialize_field("to", &self.to)?;
        s.serialize_field("value", &self.value)?;
        s.serialize_field("hash", &self.hash)?;
        s.end()
    }
}

// mod tests {
//     use super::*;
//     use std::clone;

//     const GENESIS: Block = Block {
//         hash: [0; 32],
//         parent_hash: [0; 32],
//         transactions: Vec::new(),
//     };

//     const TRANSACTIONS: [Transaction; 2] = [Transaction {}, Transaction {}];

//     fn init() -> Blockchain {
//         let mut blockchain = Blockchain::new(GENESIS);
//         blockchain.transactions = Vec::from(TRANSACTIONS);

//         blockchain
//     }

//     #[test]
//     fn create_blockchain() {
//         let blockchain = init();

//         assert_eq!(blockchain.chain[0], GENESIS);
//         assert_eq!(blockchain.transactions[0..1], TRANSACTIONS);
//     }

//     // #[test]
//     // fn create_block() {
//     //     let block = Block {
//     //         hash: ,
//     //         parent_hash: GENESIS.hash,
//     //         transactions: Vec
//     //     };

//     //     let blockchain = init();
//     //     let parent_hash = blockchain.chain[0].hash;

//     //     let mut txs: Vec<Transaction>;
//     //     txs.clone_from_slice(&blockchain.transactions[0..1]);

//     //     let mut new_block = Block::new(parent_hash, txs);

//     //     assert_eq!(block, new_block);
//     //     assert_eq!(block, new_block);
//     // }
// }
