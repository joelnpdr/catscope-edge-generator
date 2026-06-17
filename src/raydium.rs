use solana_sdk::pubkey::Pubkey;
use std::collections::VecDeque;

#[cfg(any(target_os = "wasi", target_os = "linux"))]
use crate::primitive::wasmimport::HostImport;
use crate::{
    primitive::{
        guest::GuestFilter,
        header::AccountHeader,
        tree::{FilterEdge, WEIGHT_DIRECT, WEIGHT_PROGRAM, WEIGHT_SYMLINK},
    },
    pubkey_is_blank,
};
pub struct RaydiumCLMM {
    len_poolstate: usize,
    len_ammconfig: usize,
    pub program_id: Pubkey,
}
impl RaydiumCLMM {
    pub fn new(program_id: &Pubkey) -> Self {
        Self {
            program_id: *program_id,
            len_poolstate: std::mem::size_of::<PoolState>(),
            len_ammconfig: std::mem::size_of::<AmmConfig>(),
        }
    }
}
impl GuestFilter for RaydiumCLMM {
    fn program_id_list(&self) -> Vec<Pubkey> {
        vec![self.program_id]
    }

    fn edge(&self, header: &AccountHeader, data: &[u8]) -> VecDeque<FilterEdge> {
        let mut list = VecDeque::new();
        let id = header.pubkey;
        if data.len() == self.len_poolstate {
            let ptr = data.as_ptr() as *const PoolState;
            let a = unsafe { &*ptr };

            if let Some(pubkey) = pubkey_is_blank(&a.amm_config) {
                list.push_back(FilterEdge {
                    slot: header.slot,
                    weight: WEIGHT_DIRECT,
                    from: id,
                    to: *pubkey,
                });
            }
            if let Some(pubkey) = pubkey_is_blank(&a.owner) {
                list.push_back(FilterEdge {
                    slot: header.slot,
                    weight: WEIGHT_DIRECT,
                    from: *pubkey,
                    to: id,
                });
            }
            if let Some(pubkey) = pubkey_is_blank(&a.token_vault_0) {
                list.push_back(FilterEdge {
                    slot: header.slot,
                    weight: WEIGHT_DIRECT,
                    from: id,
                    to: *pubkey,
                });
            }
            if let Some(pubkey) = pubkey_is_blank(&a.token_vault_1) {
                list.push_back(FilterEdge {
                    slot: header.slot,
                    weight: WEIGHT_DIRECT,
                    from: id,
                    to: *pubkey,
                });
            }
            if let Some(pubkey) = pubkey_is_blank(&a.observation_key) {
                list.push_back(FilterEdge {
                    slot: header.slot,
                    weight: WEIGHT_DIRECT,
                    from: id,
                    to: *pubkey,
                });
            }
        } else if data.len() == self.len_ammconfig {
            let ptr = data.as_ptr() as *const AmmConfig;
            let a = unsafe { &*ptr };
            list.push_back(FilterEdge {
                slot: header.slot,
                weight: WEIGHT_PROGRAM,
                from: self.program_id,
                to: id,
            });
            if let Some(pubkey) = pubkey_is_blank(&a.owner) {
                list.push_back(FilterEdge {
                    slot: header.slot,
                    weight: WEIGHT_SYMLINK,
                    from: *pubkey,
                    to: id,
                });
            }
            if let Some(pubkey) = pubkey_is_blank(&a.fund_owner) {
                list.push_back(FilterEdge {
                    slot: header.slot,
                    weight: WEIGHT_SYMLINK,
                    from: *pubkey,
                    to: id,
                });
            }
        }

        #[cfg(any(target_os = "wasi", target_os = "linux"))]
        HostImport::log(format!("raydium_edge - 4 - pubkey {id};"));
        list
    }
}

pub struct RaydiumAmm {
    len_amminfo: usize,
    pub program_id: Pubkey,
}

impl GuestFilter for RaydiumAmm {
    fn program_id_list(&self) -> Vec<Pubkey> {
        vec![self.program_id]
    }

    fn edge(&self, header: &AccountHeader, data: &[u8]) -> VecDeque<FilterEdge> {
        let mut list = VecDeque::new();
        let id = header.pubkey;

        #[cfg(any(target_os = "wasi", target_os = "linux"))]
        HostImport::log(format!(
            "raydium_amm_edge - pubkey {}; data len {}",
            id,
            data.len()
        ));
        let a = if self.len_amminfo == data.len() {
            let ptr = data.as_ptr() as *const AmmInfo;
            unsafe { &*ptr }
        } else {
            return list;
        };
        // program → pool
        list.push_back(FilterEdge {
            slot: header.slot,
            weight: WEIGHT_DIRECT,
            from: self.program_id,
            to: id,
        });

        // coin vault
        if let Some(pk) = pubkey_is_blank(&a.coin_vault) {
            list.push_back(FilterEdge {
                slot: header.slot,
                weight: WEIGHT_DIRECT,
                from: id,
                to: *pk,
            });
        }
        if let Some(pk) = pubkey_is_blank(&a.pc_vault) {
            list.push_back(FilterEdge {
                slot: header.slot,
                weight: WEIGHT_DIRECT,
                from: id,
                to: *pk,
            });
        }
        if let Some(pk) = pubkey_is_blank(&a.open_orders) {
            list.push_back(FilterEdge {
                slot: header.slot,
                weight: WEIGHT_DIRECT,
                from: id,
                to: *pk,
            });
        }
        if let Some(pk) = pubkey_is_blank(&a.market) {
            list.push_back(FilterEdge {
                slot: header.slot,
                weight: WEIGHT_DIRECT,
                from: id,
                to: *pk,
            });
        }
        if let Some(pk) = pubkey_is_blank(&a.target_orders) {
            list.push_back(FilterEdge {
                slot: header.slot,
                weight: WEIGHT_DIRECT,
                from: id,
                to: *pk,
            });
        }
        if let Some(pk) = pubkey_is_blank(&a.withdraw_queue) {
            list.push_back(FilterEdge {
                slot: header.slot,
                weight: WEIGHT_DIRECT,
                from: id,
                to: *pk,
            });
        }
        if let Some(pk) = pubkey_is_blank(&a.owner) {
            list.push_back(FilterEdge {
                slot: header.slot,
                weight: WEIGHT_DIRECT,
                from: *pk,
                to: id,
            });
        }

        list
    }
}

impl RaydiumAmm {
    pub fn new(program_id: &Pubkey) -> Self {
        Self {
            len_amminfo: std::mem::size_of::<AmmInfo>(),
            program_id: *program_id,
        }
    }
}

#[repr(C, packed)]
pub struct AmmInfo {
    pub status: u64, // bitmask: swap/deposit/withdraw/crank enabled
    pub nonce: u64,  // bump used to derive amm_authority
    pub order_num: u64,
    pub depth: u64,
    pub coin_decimals: u64,
    pub pc_decimals: u64,
    pub state: u64, // internal state machine
    pub reset_flag: u64,
    pub min_size: u64,
    pub vol_max_cut_ratio: u64,
    pub amount_wave: u64,
    pub coin_lot_size: u64, // mirrors OpenBook market
    pub pc_lot_size: u64,
    pub min_price_multiplier: u64,
    pub max_price_multiplier: u64,
    pub sys_decimal_value: u64,

    pub fees: Fees, // trade/protocol/fund fee rates
    pub state_data: StateData,

    // Pool-owned accounts:
    pub coin_vault: Pubkey,
    pub pc_vault: Pubkey,
    pub coin_vault_mint: Pubkey,
    pub pc_vault_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub open_orders: Pubkey,    // pool's OpenOrders on OpenBook
    pub market: Pubkey,         // OpenBook market
    pub market_program: Pubkey, // OpenBook program ID
    pub target_orders: Pubkey,
    pub withdraw_queue: Pubkey,
    pub lp_vault: Pubkey, // = pool_temp_lp
    pub owner: Pubkey,    // admin (multisig)
    pub lp_reserve: u64,
    pub padding: [u64; 3],
}

#[repr(C, packed)]
pub struct Fees {
    pub min_separate_numerator: u64,   // 5
    pub min_separate_denominator: u64, // 10_000
    pub trade_fee_numerator: u64,      // 25  → used by OpenBook integration
    pub trade_fee_denominator: u64,    // 10_000
    pub pnl_numerator: u64,            // 12  → protocol's share OF the swap fee
    pub pnl_denominator: u64,          // 100 → so 12/100 = 12% of fee, = 0.03% of volume
    pub swap_fee_numerator: u64,       // 25  → 0.25% gross swap fee
    pub swap_fee_denominator: u64,     // 10_000
}

#[repr(C, packed)]
pub struct StateData {
    pub need_take_pnl_coin: u64,
    pub need_take_pnl_pc: u64,
    pub total_pnl_pc: u64,
    pub total_pnl_coin: u64,
    pub pool_open_time: u64,
    pub punish_pc_amount: u64,
    pub punish_coin_amount: u64,
    pub orderbook_to_init_time: u64,
    pub swap_coin_in_amount: u128,
    pub swap_pc_out_amount: u128,
    pub swap_acc_pc_fee: u64,
    pub swap_pc_in_amount: u128,
    pub swap_coin_out_amount: u128,
    pub swap_acc_coin_fee: u64,
}
#[repr(C, packed)]
pub struct AmmConfig {
    pub bump: u8,
    pub index: u16, // uses "amm_config"+u16 seed

    pub owner: Pubkey,          // admin
    pub protocol_fee_rate: u32, // fraction of trade fee to protocol, denom 1e6
    pub trade_fee_rate: u32,    // trade fee in 1e6ths of volume
    pub tick_spacing: u16,      // default spacing for pools using this config
    pub fund_fee_rate: u32,     // fraction of trade fee to fund, denom 1e6
    pub padding_u32: u32,

    pub fund_owner: Pubkey,
    pub padding: [u64; 3],
}
#[repr(C, packed)]
pub struct PoolState {
    pub bump: [u8; 1],
    pub amm_config: Pubkey, // fee tier binding
    pub owner: Pubkey,      // admin (multisig)
    pub token_mint_0: Pubkey,
    pub token_mint_1: Pubkey,
    pub token_vault_0: Pubkey,
    pub token_vault_1: Pubkey,
    pub observation_key: Pubkey,

    pub mint_decimals_0: u8,
    pub mint_decimals_1: u8,
    pub tick_spacing: u16, // inherited from amm_config at init

    pub liquidity: u128,      // total active (in-range) liquidity
    pub sqrt_price_x64: u128, // Q64.64 of sqrt(price)
    pub tick_current: i32,    // current tick index

    pub padding3: u16,
    pub padding4: u16,

    // Global fee growth per unit of liquidity, Q64.64.
    pub fee_growth_global_0_x64: u128,
    pub fee_growth_global_1_x64: u128,

    // Accrued-but-not-swept protocol fees (per mint).
    pub protocol_fees_token_0: u64,
    pub protocol_fees_token_1: u64,

    // Reserved padding for future upgrades.
    pub padding5: [u128; 4],

    // Status bitmask. Bits 0-5: open-position, decrease-liquidity,
    // collect-fee, collect-reward, swap, limit-order. A set bit disables
    // the corresponding operation.
    pub status: u8,

    // Fee-collection mode (CollectFeeOn).
    //   0 = FromInput (deduct fee from the swap input — Uniswap-V3 default)
    //   1 = Token0Only (always deduct fee from token0 vault)
    //   2 = Token1Only (always deduct fee from token1 vault)
    pub fee_on: u8,
    pub padding: [u8; 6],

    // Live reward streams (up to REWARD_NUM = 3).
    pub reward_infos: [RewardInfo; 3],

    // Inline bitmap tracking initialized tick-arrays in the primary range.
    pub tick_array_bitmap: [u64; 16],

    // Reserved padding for future upgrades.
    pub padding6: [u64; 4],

    pub fund_fees_token_0: u64,
    pub fund_fees_token_1: u64,

    pub open_time: u64, // currently disabled by the program
    pub recent_epoch: u64,

    // Per-pool dynamic-fee state. Zero-valued unless the pool was
    // created with `enable_dynamic_fee = true` via create_customizable_pool.
    pub dynamic_fee_info: DynamicFeeInfo,

    // Reserved for future upgrades.
    pub padding1: [u64; 14],
    pub padding2: [u64; 32],
}

#[repr(C, packed)]
pub struct RewardInfo {
    /// Reward state
    pub reward_state: u8,
    /// Reward open time
    pub open_time: u64,
    /// Reward end time
    pub end_time: u64,
    /// Reward last update time
    pub last_update_time: u64,
    /// Q64.64 number indicates how many tokens per second are earned per unit of liquidity.
    pub emissions_per_second_x64: u128,
    /// The total amount of reward emitted
    pub reward_total_emitted: u64,
    /// The total amount of claimed reward
    pub reward_claimed: u64,
    /// Reward token mint.
    pub token_mint: Pubkey,
    /// Reward vault token account.
    pub token_vault: Pubkey,
    /// The owner that has permission to set reward param
    pub authority: Pubkey,
    /// Q64.64 number that tracks the total tokens earned per unit of liquidity since the reward
    /// emissions were turned on.
    pub reward_growth_global_x64: u128,
}

#[repr(C, packed)]
pub struct DynamicFeeInfo {
    /// Period that determines the high frequency trading time window (in seconds).
    pub filter_period: u16,
    /// Period that determines when the dynamic fee starts to decrease (in seconds).
    pub decay_period: u16,
    /// Dynamic fee rate decrement rate, used for volatility reference decay.
    pub reduction_factor: u16,
    /// Factor used to scale the dynamic fee component in the fee rate calculation.
    pub dynamic_fee_control: u32,
    /// Maximum value for the volatility accumulator, used to cap the dynamic fee rate.
    pub max_volatility_accumulator: u32,

    /// Active tick spacing index at the last reference update.
    pub tick_spacing_index_reference: i32,
    /// Volatility reference value, stores the decayed volatility accumulator.
    pub volatility_reference: u32,
    /// Volatility accumulator, used to calculate the dynamic fee rate.
    pub volatility_accumulator: u32,
    /// Last timestamp (block time) when `volatility_reference` and `tick_spacing_index_reference` were updated.
    pub last_update_timestamp: u64,
    /// Reserved for future upgrades.
    pub padding: [u8; 46],
}

#[repr(C, packed)]
pub struct PositionRewardInfo {
    pub growth_inside_last_x64: u128,
    pub reward_amount_owed: u64,
}

// ─── Raydium CLMM discriminators ─────────────────────────────────────────────

pub fn discriminator_amm_config() -> [u8; 8] {
    [218, 244, 33, 104, 203, 203, 43, 111]
}
pub fn discriminator_observation_state() -> [u8; 8] {
    [122, 174, 197, 53, 129, 9, 165, 132]
}
pub fn discriminator_operation_state() -> [u8; 8] {
    [19, 236, 58, 237, 81, 222, 183, 252]
}
pub fn discriminator_personal_position_state() -> [u8; 8] {
    [70, 111, 150, 126, 230, 15, 25, 117]
}
pub fn discriminator_pool_state() -> [u8; 8] {
    [247, 237, 227, 245, 215, 195, 222, 70]
}
pub fn discriminator_protocol_position_state() -> [u8; 8] {
    [100, 226, 145, 99, 146, 218, 160, 106]
}
pub fn discriminator_support_mint_associated() -> [u8; 8] {
    [134, 40, 183, 79, 12, 112, 162, 53]
}
pub fn discriminator_tick_array_bitmap_extension() -> [u8; 8] {
    [60, 150, 36, 219, 97, 128, 139, 153]
}
pub fn discriminator_tick_array_state() -> [u8; 8] {
    [192, 155, 85, 205, 49, 249, 129, 42]
}
