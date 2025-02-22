use crate::packed::{DepositRequest, Script};
use crate::{
    bytes::Bytes,
    packed::{CellInput, CellOutput, OutPoint},
};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct CellInfo {
    pub out_point: OutPoint,
    pub output: CellOutput,
    pub data: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellStatus {
    Live,
    Dead,
    Unknown,
}

impl Default for CellStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Default)]
pub struct CellWithStatus {
    pub cell: Option<CellInfo>,
    pub status: CellStatus,
}

#[derive(Debug, Clone)]
pub struct InputCellInfo {
    pub input: CellInput,
    pub cell: CellInfo,
}

#[derive(Debug, Clone)]
pub struct CollectedCustodianCells {
    pub cells_info: Vec<CellInfo>,
    pub capacity: u128,
    pub sudt: HashMap<[u8; 32], (u128, Script)>,
}

impl Default for CollectedCustodianCells {
    fn default() -> Self {
        CollectedCustodianCells {
            cells_info: Default::default(),
            capacity: 0,
            sudt: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct WithdrawalsAmount {
    pub capacity: u128,
    pub sudt: HashMap<[u8; 32], u128>,
}

impl Default for WithdrawalsAmount {
    fn default() -> Self {
        WithdrawalsAmount {
            capacity: 0,
            sudt: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TxStatus {
    /// Status "pending". The transaction is in the pool, and not proposed yet.
    Pending,
    /// Status "proposed". The transaction is in the pool and has been proposed.
    Proposed,
    /// Status "committed". The transaction has been committed to the canonical chain.
    Committed,
}

#[derive(Debug, Clone)]
pub struct DepositInfo {
    pub request: DepositRequest,
    pub cell: CellInfo,
}

#[derive(Debug, Clone, Default)]
pub struct SUDTStat {
    pub total_amount: u128,
    pub finalized_amount: u128,
    pub cells_count: usize,
}

#[derive(Debug, Clone)]
pub struct CustodianStat {
    pub total_capacity: u128,
    pub finalized_capacity: u128,
    pub cells_count: usize,
    pub ckb_cells_count: usize,
    pub sudt_stat: HashMap<ckb_types::packed::Script, SUDTStat>,
}
