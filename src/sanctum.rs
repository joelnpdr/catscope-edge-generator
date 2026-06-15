//! Edge filter for the Sanctum S Controller (INF / Infinity) program.
//!
//! # Program
//!
//! The Sanctum S Controller (`5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx`) is
//! **not** an Anchor program — it uses `bytemuck::Pod` with no 8-byte discriminator.
//! Accounts are identified by comparing their pubkey against precomputed PDAs.
//!
//! # Account layout — PoolState (176 bytes, no discriminator)
//!
//! ```text
//! offset   size  field
//! ──────   ────  ─────────────────────────────────────────
//!  0        8    total_sol_value: u64
//!  8        2    trading_protocol_fee_bps: u16
//! 10        2    lp_protocol_fee_bps: u16
//! 12        1    version: u8
//! 13        1    is_disabled: u8
//! 14        1    is_rebalancing: u8
//! 15        1    padding: [u8; 1]
//! 16       32    admin: Pubkey
//! 48       32    rebalance_authority: Pubkey
//! 80       32    protocol_fee_beneficiary: Pubkey
//! 112      32    pricing_program: Pubkey
//! 144      32    lp_token_mint: Pubkey   ← INF token mint
//! ```
//!
//! # Account layout — LstState (80 bytes per entry, no discriminator)
//!
//! The LST state list account is a flat array of `LstState` entries:
//!
//! ```text
//! offset (within entry)  size  field
//! ─────────────────────  ────  ─────────────────────────────────
//!  0                      1    is_input_disabled: u8
//!  1                      1    pool_reserves_bump: u8
//!  2                      1    protocol_fee_accumulator_bump: u8
//!  3                      5    padding: [u8; 5]
//!  8                      8    sol_value: u64
//! 16                     32    mint: Pubkey   ← LST mint address
//! 48                     32    sol_value_calculator: Pubkey
//! ```
//!
//! # PDAs
//!
//! ```text
//! pool_state_pda     = PDA([b"state"],          program_id)
//! lst_state_list_pda = PDA([b"lst-state-list"],  program_id)
//! ```

use std::collections::VecDeque;

use solana_sdk::{pubkey::Pubkey, system_program::ID as system_id};

#[cfg(any(target_os = "wasi", target_os = "linux"))]
use crate::primitive::wasmimport::HostImport;
use crate::primitive::{
    guest::GuestFilter,
    header::AccountHeader,
    tree::{FilterEdge, WEIGHT_DIRECT},
};

/// Known mainnet program ID for the Sanctum S Controller (INF).
pub const S_CONTROLLER_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx");

const POOL_STATE_SIZE: usize = 176;
const LST_STATE_SIZE: usize = 80;

/// Byte offset of `lp_token_mint` within PoolState.
const OFF_PS_LP_TOKEN_MINT: usize = 144;

/// Byte offset of `mint` within each LstState entry.
const OFF_LS_MINT: usize = 16;

pub struct Sanctum {
    pub program_id: Pubkey,
    /// PDA([b"state"], program_id)
    pool_state_pda: Pubkey,
    /// PDA([b"lst-state-list"], program_id)
    lst_state_list_pda: Pubkey,
}

impl GuestFilter for Sanctum {
    fn program_id_list(&self) -> Vec<Pubkey> {
        vec![self.program_id]
    }

    fn edge(&self, header: &AccountHeader, data: &[u8]) -> VecDeque<FilterEdge> {
        let mut list = VecDeque::new();
        let id = header.pubkey;
        let pubkey_len = std::mem::size_of::<Pubkey>();

        #[cfg(any(target_os = "wasi", target_os = "linux"))]
        HostImport::log(format!(
            "sanctum_edge - pubkey {}; data len {}",
            id,
            data.len()
        ));

        if id == self.pool_state_pda && data.len() == POOL_STATE_SIZE {
            // program → pool_state
            list.push_back(FilterEdge {
                slot: header.slot,
                weight: WEIGHT_DIRECT,
                from: self.program_id,
                to: id,
            });

            // pool_state → lp_token_mint (the INF token)
            if data.len() >= OFF_PS_LP_TOKEN_MINT + pubkey_len {
                let mint_pk = Pubkey::try_from(
                    &data[OFF_PS_LP_TOKEN_MINT..OFF_PS_LP_TOKEN_MINT + pubkey_len],
                )
                .unwrap();
                if mint_pk != system_id {
                    list.push_back(FilterEdge {
                        slot: header.slot,
                        weight: WEIGHT_DIRECT,
                        from: id,
                        to: mint_pk,
                    });
                }
            }
        } else if id == self.lst_state_list_pda
            && !data.is_empty()
            && data.len() % LST_STATE_SIZE == 0
        {
            // pool_state → lst_state_list
            list.push_back(FilterEdge {
                slot: header.slot,
                weight: WEIGHT_DIRECT,
                from: self.pool_state_pda,
                to: id,
            });

            // lst_state_list → each LST mint
            let n = data.len() / LST_STATE_SIZE;
            for i in 0..n {
                let mint_off = i * LST_STATE_SIZE + OFF_LS_MINT;
                if data.len() >= mint_off + pubkey_len {
                    let mint_pk =
                        Pubkey::try_from(&data[mint_off..mint_off + pubkey_len]).unwrap();
                    if mint_pk != system_id {
                        list.push_back(FilterEdge {
                            slot: header.slot,
                            weight: WEIGHT_DIRECT,
                            from: id,
                            to: mint_pk,
                        });
                    }
                }
            }
        }

        list
    }
}

impl Sanctum {
    pub fn new(program_id: &Pubkey) -> Self {
        let pool_state_pda =
            Pubkey::find_program_address(&[b"state"], program_id).0;
        let lst_state_list_pda =
            Pubkey::find_program_address(&[b"lst-state-list"], program_id).0;
        Self {
            program_id: *program_id,
            pool_state_pda,
            lst_state_list_pda,
        }
    }
}
