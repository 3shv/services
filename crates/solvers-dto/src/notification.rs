use {
    super::serialize,
    chrono::{DateTime, Utc},
    number::serialization::HexOrDecimalU256,
    serde::{Deserialize, Serialize},
    serde_with::{DisplayFromStr, serde_as},
    std::collections::BTreeSet,
    web3::types::{AccessList, H160, H256, U256},
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub auction_id: Option<i64>,
    pub solution_id: Option<SolutionId>,
    #[serde(flatten)]
    pub kind: Kind,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SolutionId {
    Single(u64),
    Merged(Vec<u64>),
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Kind {
    Timeout,
    EmptySolution,
    DuplicatedSolutionId,
    #[serde(rename_all = "camelCase")]
    SimulationFailed {
        block: BlockNo,
        tx: Tx,
        succeeded_once: bool,
    },
    InvalidClearingPrices,
    #[serde(rename_all = "camelCase")]
    MissingPrice {
        token_address: H160,
    },
    InvalidExecutedAmount,
    NonBufferableTokensUsed {
        tokens: BTreeSet<H160>,
    },
    SolverAccountInsufficientBalance {
        #[serde_as(as = "HexOrDecimalU256")]
        required: U256,
    },
    Success {
        transaction: H256,
    },
    Revert {
        transaction: H256,
    },
    DriverError {
        reason: String,
    },
    Cancelled,
    Expired,
    Fail,
    PostprocessingTimedOut,
    Banned {
        reason: BanReason,
        until: DateTime<Utc>,
    },
}

type BlockNo = u64;

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tx {
    pub from: H160,
    pub to: H160,
    #[serde_as(as = "serialize::Hex")]
    pub input: Vec<u8>,
    #[serde_as(as = "HexOrDecimalU256")]
    pub value: U256,
    pub access_list: AccessList,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "reason")]
pub enum BanReason {
    UnsettledConsecutiveAuctions,
    HighSettleFailureRate,
}
