use std::cmp;
use std::collections::HashSet;

use primitive_types::U256;

use crate::check_difficulty;

use super::Block;
use super::Hash;
use super::{now, Transaction, TxOutput};

#[derive(Debug)]
pub enum BlockChainError {
    ProofOfWorkError(String),
    NotACoinBaseError(String),
    InvalidTransactionError(String),
    InsufficientFundsError(String),
    InputNotSpendableError(String),
    DoubleSpendingError(String),
}

pub struct Blockchain {
    blocks: Vec<Block>,
    transaction_pool: Vec<Transaction>,
    pub unspent_output: HashSet<Hash>,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain {
            blocks: vec![],
            transaction_pool: vec![],
            unspent_output: HashSet::new(),
        }
    }

    pub fn add_transaction_to_pool(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), BlockChainError> {
        // verify transaction
        match self.verify_transaction(&transaction) {
            Ok(()) => println!("transaction verified"),
            Err(e) => {
                println!("{:?}", e);
                return Err(e);
            }
        }

        //TODO complete the validation process ( see spec document)
        self.transaction_pool.push(transaction);
        Ok(())
    }

    pub fn create_candidate_block(
        &mut self,
        transactions_count: usize,
        miner_address: String,
        difficulty: U256,
    ) -> Block {
        let mut candidate_index: u32 = 0;
        let mut previous_hash: Hash = Vec::new();
        if let Some(latest_block) = self.blocks.last().cloned() {
            candidate_index = latest_block.index;
            previous_hash = latest_block.hash;
        }
        //Get transactions from pool up to transactions count
        let pool_size = self.transaction_pool.len();
        let block_transaction_count = cmp::min(pool_size, transactions_count);
        let mut transactions: Vec<Transaction> = Vec::new();

        // Add coinbase transaction
        let coinbase = Transaction {
            inputs: vec![],
            outputs: vec![TxOutput {
                address: miner_address,
                value: 50.0,
            }],
            timestamp: now(),
        };
        transactions.push(coinbase);

        transactions.extend(
            self.transaction_pool
                .drain(..block_transaction_count)
                .collect::<Vec<Transaction>>(),
        );
        Block::new(
            candidate_index + 1,
            now(),
            previous_hash,
            transactions,
            difficulty,
        )
    }

    pub fn aggregate_mined_block(&mut self, block: Block) -> Result<(), BlockChainError> {
        if !check_difficulty(&block.hash, block.difficulty) {
            return Err(BlockChainError::ProofOfWorkError(String::from(
                "Block is not correctly mined",
            )));
        }
        if let Some((coinbase, transactions)) = block.transactions.split_first() {
            if !coinbase.is_coinbase() {
                return Err(BlockChainError::NotACoinBaseError(String::from(
                    "First transaction in block must be a coinbase.",
                )));
            }

            let mut output_spent = Vec::new();
            let mut output_created = Vec::new();
            // Add coinbase output
            output_created.extend(coinbase.output_hashes());

            for transaction in transactions {
                match self.verify_transaction(transaction) {
                    Ok(()) => println!("transaction verified"),
                    Err(e) => return Err(e),
                }
                output_spent.extend(transaction.input_hashes());
                output_created.extend(transaction.output_hashes());
            }

            // Update unspent output vector
            self.unspent_output
                .retain(|output| !output_spent.contains(output));
            self.unspent_output.extend(output_created);
            self.blocks.push(block);
        }
        Ok(())
    }

    fn verify_transaction(&self, transaction: &Transaction) -> Result<(), BlockChainError> {
        // check if transaction is spendable
        if !transaction.is_spendable() {
            return Err(BlockChainError::InsufficientFundsError(String::from(
                "Transaction output is grater than input.",
            )));
        }
        // check inputs are valid (unspent output in block)
        let input_hashes = transaction.input_hashes();
        for hash in input_hashes {
            if !self.unspent_output.contains(&hash) {
                return Err(BlockChainError::InputNotSpendableError(String::from(
                    "Input is not spendable.",
                )));
            }
            let tx_pool_hashes = self
                .transaction_pool
                .iter()
                .flat_map(|transaction| transaction.input_hashes())
                .collect::<HashSet<Hash>>();
            if tx_pool_hashes.contains(&hash) {
                return Err(BlockChainError::DoubleSpendingError(String::from(
                    "Double spending attempt.",
                )));
            }
        }
        return Ok(());
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }
}

//Testing

#[cfg(test)]
mod tests {
    use primitive_types::U256;

    use crate::{now, Blockchain, Hashable, Transaction, TxOutput};

    #[test]
    fn add_transaction_to_pool() {
        // Given
        let mut blockchain: Blockchain = Blockchain::new();
        let unspent_outputs = vec![
            TxOutput {
                address: String::from("Alice"),
                value: 10.0,
            },
            TxOutput {
                address: String::from("Alice"),
                value: 20.0,
            },
        ];
        blockchain
            .unspent_output
            .extend(unspent_outputs.iter().map(|output| output.hash()));
        let transaction = Transaction {
            inputs: unspent_outputs,
            outputs: vec![
                TxOutput {
                    address: String::from("Bob"),
                    value: 25.0,
                },
                TxOutput {
                    address: String::from("Bob"),
                    value: 4.995,
                },
            ],
            timestamp: now(),
        };

        blockchain.add_transaction_to_pool(transaction).unwrap();
        assert_eq!(1, blockchain.transaction_pool.len());
        assert_eq!(2, blockchain.unspent_output.len());
    }

    #[test]
    fn should_create_candidate_block() {
        let mut blockchain: Blockchain = Blockchain::new();
        let block = blockchain.create_candidate_block(5, String::from("Alice"), U256::max_value());
        println!("{:?}", block);
    }
}
