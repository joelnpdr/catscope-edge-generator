use std::collections::VecDeque;

use solana_sdk::{pubkey::Pubkey, system_program};

#[cfg(any(target_os = "wasi", target_os = "linux"))]
use crate::primitive::wasmimport::HostImport;
use crate::primitive::{
    common::match_discriminator,
    guest::GuestFilter,
    header::AccountHeader,
    tree::{FilterEdge, WEIGHT_DIRECT},
};

pub struct Kamino {
    d_lending_market: [u8; 8],
    d_reserve: [u8; 8],
    pub program_id: Pubkey,
}

impl GuestFilter for Kamino {
    fn program_id_list(&self) -> Vec<Pubkey> {
        vec![self.program_id]
    }

    fn edge(&self, header: &AccountHeader, data: &[u8]) -> VecDeque<FilterEdge> {
        let mut list = VecDeque::new();
        let id = header.pubkey;
        let pubkey_len = std::mem::size_of::<Pubkey>();

        #[cfg(any(target_os = "wasi", target_os = "linux"))]
        HostImport::log(format!(
            "kamino_edge - pubkey {}; data len {}",
            id,
            data.len()
        ));

        if match_discriminator(&self.d_lending_market, data) {
            // program → lending_market
            list.push_back(FilterEdge {
                slot: header.slot,
                weight: WEIGHT_DIRECT,
                from: self.program_id,
                to: id,
            });
        } else if match_discriminator(&self.d_reserve, data) {
            // lending_market → reserve (offset 32)
            if data.len() >= 32 + pubkey_len {
                let lm_pk =
                    Pubkey::try_from(&data[32..32 + pubkey_len]).unwrap();
                if lm_pk != system_program::ID {
                    list.push_back(FilterEdge {
                        slot: header.slot,
                        weight: WEIGHT_DIRECT,
                        from: lm_pk,
                        to: id,
                    });
                }
            }
            // reserve → supply_vault (offset 160)
            if data.len() >= 160 + pubkey_len {
                let vault_pk =
                    Pubkey::try_from(&data[160..160 + pubkey_len]).unwrap();
                if vault_pk != system_program::ID {
                    list.push_back(FilterEdge {
                        slot: header.slot,
                        weight: WEIGHT_DIRECT,
                        from: id,
                        to: vault_pk,
                    });
                }
            }
            // reserve → fee_vault (offset 192)
            if data.len() >= 192 + pubkey_len {
                let fee_pk =
                    Pubkey::try_from(&data[192..192 + pubkey_len]).unwrap();
                if fee_pk != system_program::ID {
                    list.push_back(FilterEdge {
                        slot: header.slot,
                        weight: WEIGHT_DIRECT,
                        from: id,
                        to: fee_pk,
                    });
                }
            }
        }

        list
    }
}

impl Kamino {
    pub fn new(program_id: &Pubkey) -> Self {
        Self {
            program_id: *program_id,
            d_lending_market: lending_market_discriminator(),
            d_reserve: reserve_discriminator(),
        }
    }
}

/// sha256("account:LendingMarket")[..8]
pub fn lending_market_discriminator() -> [u8; 8] {
    [246, 114, 50, 98, 72, 157, 28, 120]
}

/// sha256("account:Reserve")[..8]
pub fn reserve_discriminator() -> [u8; 8] {
    [43, 242, 204, 202, 26, 247, 59, 127]
}
