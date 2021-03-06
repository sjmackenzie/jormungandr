use crate::{
    blockcfg::{BlockBuilder, HeaderContentEvalContext, Ledger, LedgerParameters},
    fragment::{FragmentId, Status},
};

use super::logs::internal::Logs;
use super::pool::internal::Pool;

pub enum SelectionOutput {
    Commit { fragment_id: FragmentId },
    RequestSmallerFee,
    RequestSmallerSize,
    Reject { reason: String },
}

pub trait FragmentSelectionAlgorithm {
    fn select(
        &mut self,
        ledger: &Ledger,
        ledger_params: &LedgerParameters,
        metadata: &HeaderContentEvalContext,
        logs: &mut Logs,
        pool: &mut Pool,
    );

    fn finalize(self) -> BlockBuilder;
}

pub struct OldestFirst {
    builder: BlockBuilder,
    max_per_block: usize,
}

impl OldestFirst {
    pub fn new(max_per_block: usize) -> Self {
        OldestFirst {
            builder: BlockBuilder::new(),
            max_per_block,
        }
    }
}

impl FragmentSelectionAlgorithm for OldestFirst {
    fn finalize(self) -> BlockBuilder {
        self.builder
    }

    fn select(
        &mut self,
        ledger: &Ledger,
        ledger_params: &LedgerParameters,
        metadata: &HeaderContentEvalContext,
        logs: &mut Logs,
        pool: &mut Pool,
    ) {
        let mut total = 0usize;

        while let Some(id) = pool.entries_by_time.pop_front() {
            if total >= self.max_per_block {
                break;
            }

            let fragment = pool.remove(&id).unwrap();

            match ledger.apply_fragment(ledger_params, &fragment, metadata) {
                Ok(_) => {
                    self.builder.message(fragment);

                    logs.modify(
                        &id,
                        Status::InABlock {
                            date: metadata.block_date,
                        },
                    );

                    total += 1;
                }
                Err(error) => logs.modify(
                    &id,
                    Status::Rejected {
                        reason: error.to_string(),
                    },
                ),
            }
        }
    }
}
