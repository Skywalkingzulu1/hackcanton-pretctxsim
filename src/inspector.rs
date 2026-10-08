use revm::{
    interpreter::Interpreter,
    EvmContext, Inspector, Database, DatabaseRef,
    primitives::{Address, U256, B256, TxKind, Log},
};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct BalanceChange {
    pub address: String,
    pub pre_balance: String,
    pub post_balance: String,
    pub delta: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AllowanceChange {
    pub owner: String,
    pub spender: String,
    pub token: String,
    pub pre_allowance: String,
    pub post_allowance: String,
    pub delta: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct TokenTransfer {
    pub token: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub token_type: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct DelegationChange {
    pub address: String,
    pub old_code_hash: Option<String>,
    pub new_code_hash: Option<String>,
    pub delegated_to: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct StorageDiff {
    pub address: String,
    pub slot: String,
    pub pre_value: String,
    pub post_value: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct EmittedLog {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct EVMStructuralAnalysis {
    pub delegatecalls_detected: usize,
    pub storage_slots_written: usize,
    pub selfdestruct_triggered: bool,
    pub eth_transferred_wei: String,
    pub balance_changes: Vec<BalanceChange>,
    pub allowance_changes: Vec<AllowanceChange>,
    pub token_transfers: Vec<TokenTransfer>,
    pub delegation_changes: Vec<DelegationChange>,
    pub storage_diffs: Vec<StorageDiff>,
    pub logs_emitted: Vec<EmittedLog>,
}

pub struct RiskInspector {
    pub analysis: EVMStructuralAnalysis,
    pre_balances: HashMap<Address, U256>,
    pre_code_hashes: HashMap<Address, B256>,
    pre_storage: HashMap<(Address, U256), U256>,
    pre_allowances: HashMap<(Address, Address, Address), U256>,
}

/// Parse an EIP-7702 delegation designator.
///
/// A delegated EOA's code is exactly: 0xEF0100 || address (23 bytes).
/// Returns the delegated-to address, or None if the code is not a designator.
/// (The previous check tested only the EF01 prefix and sliced bytes 2..22,
/// producing a misaligned address.)
pub fn parse_delegation_designator(code: &[u8]) -> Option<Address> {
    if code.len() < 23 || code[0] != 0xEF || code[1] != 0x01 || code[2] != 0x00 {
        return None;
    }
    Some(Address::from_slice(&code[3..23]))
}

impl Default for RiskInspector {
    fn default() -> Self {
        Self {
            analysis: EVMStructuralAnalysis::default(),
            pre_balances: HashMap::new(),
            pre_code_hashes: HashMap::new(),
            pre_storage: HashMap::new(),
            pre_allowances: HashMap::new(),
        }
    }
}

impl<DB: Database + DatabaseRef> Inspector<DB> for RiskInspector {
    fn initialize_interp(&mut self, _interp: &mut Interpreter, context: &mut EvmContext<DB>) {
        let tx = &context.env.tx;
        let from = tx.caller;
        let to = match tx.transact_to {
            TxKind::Call(addr) => addr,
            TxKind::Create => Address::ZERO,
        };

        let db = &context.db;
        if let Ok(Some(acc)) = db.basic_ref(from) {
            self.pre_balances.insert(from, acc.balance);
        }
        if to != Address::ZERO {
            if let Ok(Some(acc)) = db.basic_ref(to) {
                self.pre_balances.insert(to, acc.balance);
                // Store code hash for delegation detection
                self.pre_code_hashes.insert(to, acc.code_hash);

                // EIP-7702: if the target already carries a delegation
                // designator, flag it at load time so the alert fires for a
                // delegated EOA (previously this only triggered on a
                // mid-simulation code-hash change, which never happens for a
                // plain eth_sendTransaction).
                if let Ok(code) = db.code_by_hash_ref(acc.code_hash) {
                    let bytes = code.original_bytes();
                    if let Some(delegated_to) = parse_delegation_designator(&bytes) {
                        self.analysis.delegation_changes.push(DelegationChange {
                            address: format!("{:#x}", to),
                            old_code_hash: None,
                            new_code_hash: Some(format!("{:#x}", acc.code_hash)),
                            delegated_to: Some(format!("{:#x}", delegated_to)),
                        });
                    }
                }
            }
        }
    }

    fn step(&mut self, interp: &mut Interpreter, _context: &mut EvmContext<DB>) {
        let opcode = interp.current_opcode();
        match opcode {
            0xF4 => self.analysis.delegatecalls_detected += 1,
            0x55 => self.analysis.storage_slots_written += 1,
            0xFF => self.analysis.selfdestruct_triggered = true,
            _ => {}
        }
    }

    fn log(&mut self, _interp: &mut Interpreter, _context: &mut EvmContext<DB>, log: &Log) {
        self.analysis.logs_emitted.push(EmittedLog {
            address: format!("{:#x}", log.address),
            topics: log.topics().iter().map(|t| format!("{:#x}", t)).collect(),
            data: format!("0x{}", hex::encode(log.data.data.as_ref())),
        });
    }
}

impl RiskInspector {
    pub fn capture_post_state<DB: Database + DatabaseRef>(&mut self, context: &mut EvmContext<DB>) {
        let tx = &context.env.tx;
        let from = tx.caller;
        let to = match tx.transact_to {
            TxKind::Call(addr) => addr,
            _ => Address::ZERO,
        };

        let db = &context.db;
        for addr in [from, to] {
            if addr == Address::ZERO { continue; }
            if let Ok(Some(acc)) = db.basic_ref(addr) {
                let post = acc.balance;
                let pre = self.pre_balances.remove(&addr).unwrap_or_default();
                if pre != post {
                    let delta = if post > pre { format!("+{}", post - pre) } else { format!("-{}", pre - post) };
                    self.analysis.balance_changes.push(BalanceChange {
                        address: format!("{:#x}", addr),
                        pre_balance: pre.to_string(),
                        post_balance: post.to_string(),
                        delta,
                    });
                }
            }
        }

        // Allowance tracking requires pre-populating storage slots - simplified for now
        // let allowances_to_check: Vec<_> = self.pre_allowances.keys().cloned().collect();
        // for (token, owner, spender) in allowances_to_check {
        //     if let Ok(storage) = db.storage_ref(token, spender) {
        //         let post: U256 = storage;
        //         let pre = self.pre_allowances.remove(&(token, owner, spender)).unwrap_or_default();
        //         if pre != post {
        //             let delta = if post > pre { format!("+{}", post - pre) } else { format!("-{}", pre - post) };
        //             self.analysis.allowance_changes.push(AllowanceChange {
        //                 owner: format!("{:#x}", owner),
        //                 spender: format!("{:#x}", spender),
        //                 token: format!("{:#x}", token),
        //                 pre_allowance: pre.to_string(),
        //                 post_allowance: post.to_string(),
        //                 delta,
        //             });
        //         }
        //     }
        // }
    }

    pub fn detect_delegations<DB: Database + DatabaseRef>(&mut self, context: &mut EvmContext<DB>) {
        let tx = &context.env.tx;
        let to = match tx.transact_to {
            TxKind::Call(addr) => addr,
            _ => return,
        };

        let db = &context.db;
        if let Ok(Some(acc)) = db.basic_ref(to) {
            let new_hash = acc.code_hash;
            if let Some(old_hash) = self.pre_code_hashes.get(&to) {
                if *old_hash != new_hash {
                    // Fetch the new code to check for EIP-7702 delegation
                    let delegated_to = if let Ok(code) = db.code_by_hash_ref(new_hash) {
                        parse_delegation_designator(code.original_bytes().as_ref())
                            .map(|addr| format!("{:#x}", addr))
                    } else { None };

                    self.analysis.delegation_changes.push(DelegationChange {
                        address: format!("{:#x}", to),
                        old_code_hash: Some(format!("{:#x}", old_hash)),
                        new_code_hash: Some(format!("{:#x}", new_hash)),
                        delegated_to,
                    });
                }
            }
        }
    }
}

const ERC20_TRANSFER: &[u8] = &[0xdd, 0xf2, 0x52, 0xad];
const ERC20_APPROVAL: &[u8] = &[0x8c, 0x5b, 0xe1, 0xa5];
const ERC721_TRANSFER: &[u8] = &[0xdd, 0xf2, 0x52, 0xad];
const ERC1155_SINGLE: &[u8] = &[0xc3, 0xd5, 0x81, 0xb9];
const ERC1155_BATCH: &[u8] = &[0x4a, 0x39, 0x3e, 0xe0];

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const DELEGATEE: &str = "0x90F79bf6EB2c4f870365E785982E1f101E93b906";

    fn designator_bytes(delegatee: Address) -> Vec<u8> {
        let mut v = vec![0xEF, 0x01, 0x00];
        v.extend_from_slice(delegatee.as_slice());
        v
    }

    #[test]
    fn parses_valid_designator() {
        let delegatee = Address::from_str(DELEGATEE).unwrap();
        let code = designator_bytes(delegatee);
        assert_eq!(code.len(), 23);
        assert_eq!(parse_delegation_designator(&code), Some(delegatee));
    }

    #[test]
    fn rejects_short_input() {
        let delegatee = Address::from_str(DELEGATEE).unwrap();
        let mut code = designator_bytes(delegatee);
        code.pop(); // 22 bytes: one short
        assert_eq!(parse_delegation_designator(&code), None);
        assert_eq!(parse_delegation_designator(&[0xEF, 0x01]), None);
        assert_eq!(parse_delegation_designator(&[]), None);
    }

    #[test]
    fn rejects_wrong_prefix() {
        let delegatee = Address::from_str(DELEGATEE).unwrap();
        let mut code = designator_bytes(delegatee);
        code[0] = 0xEF;
        code[1] = 0x02; // EF02: not a designator
        assert_eq!(parse_delegation_designator(&code), None);

        // Regular contract code starting with 0x60 (PUSH1) must not match.
        let contract = vec![0x60, 0x80, 0x60, 0x40, 0x52];
        assert_eq!(parse_delegation_designator(&contract), None);
    }

    #[test]
    fn accepts_exactly_23_bytes_and_nothing_shorter() {
        let delegatee = Address::from_str(DELEGATEE).unwrap();
        let code = designator_bytes(delegatee);
        for n in 0..=23usize {
            let truncated = &code[..n];
            let expected = if n == 23 { Some(delegatee) } else { None };
            assert_eq!(parse_delegation_designator(truncated), expected, "failed at len={}", n);
        }
    }
}