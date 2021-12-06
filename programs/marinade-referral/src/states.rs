use anchor_lang::prelude::*;

//-----------------------------------------------------
///marinade-referral-program PDA
#[account]
pub struct GlobalState {
    // Authority (admin address)
    pub admin_account: Pubkey,
    // payment token mint (normally mSOL mint)
    pub payment_mint: Pubkey,
}

//-----------------------------------------------------
///referral PDA
#[account]
pub struct ReferralState {
    // Partner name
    pub partner_name: String, //max-length 10 bytes

    // partner Beneficiary account (native account)
    pub partner_account: Pubkey,
    // token account where to make payment (ATA mSOL address for partner_account)
    pub token_partner_account: Pubkey,

    // Transfer-periodicity-seconds (u32 amount of seconds, default: a month)
    pub transfer_duration: u32,
    // Last transfer to partner timestamp (u64, unix timestamp)
    pub last_transfer_time: i64,

    // accumulated deposit-sol amount (SOL, u64)
    pub deposit_sol_amount: u64,
    // accumulated count of deposit-sol operations (u64, for stats/monitoring)
    pub deposit_sol_operations: u64,

    // accumulated deposit-stake-account amount (SOL, u64)
    pub deposit_stake_account_amount: u64,
    // accumulated count of deposit-stake-account operations (u64, for stats/monitoring)
    pub deposit_stake_account_operations: u64,

    // accumulated liquid-unstake treasury fees (mSOL, u64)
    pub liq_unstake_msol_fees: u64,
    // accumulated liquid-unstake amount (mSOL, u64)
    pub liq_unstake_amount: u64,
    // accumulated count of unstake operations (u64, for stats/monitoring)
    pub liq_unstake_operations: u64,

    // accumulated delayed-unstake amount (mSOL, u64)
    pub delayed_unstake_amount: u64,
    // accumulated count of delayed-unstake operations (u64, for stats/monitoring)
    pub delayed_unstake_operations: u64,

    // Base % cut for the partner (basis points, default 1000 => 10%)
    pub base_fee: u32,
    // Max % cut for the partner (basis points, default 1000 => 10%)
    pub max_fee: u32,
    // Net Stake target for the max % (for example 100K SOL)
    pub max_net_stake: u64,

    // emergency-pause flag (bool)
    pub pause: bool,
}

impl ReferralState {
    pub fn reset_liq_unstake_accumulators(&mut self) {
        self.deposit_sol_amount = 0;
        self.deposit_sol_operations = 0;
        self.liq_unstake_msol_fees = 0;
        self.liq_unstake_amount = 0;
        self.liq_unstake_operations = 0;
    }

    pub fn reset_del_unstake_accumulators(&mut self) {
        self.deposit_stake_account_amount = 0;
        self.deposit_stake_account_operations = 0;
        self.delayed_unstake_amount = 0;
        self.delayed_unstake_operations = 0;
    }

    pub fn get_liq_unstake_share_amount(&self) -> u64 {

        let mut net_stake = 0;

        // more deposited than unstaked
        if self.deposit_sol_amount > self.liq_unstake_amount {
            net_stake = self
                .deposit_sol_amount
                .wrapping_sub(self.liq_unstake_amount);
        }

        let share_fee_bp = if net_stake == 0 {
            self.base_fee  // minimum
        } else if net_stake > self.max_net_stake {
            self.max_fee // max
        } else {
            let delta = self
                .max_fee
                .wrapping_sub(self.base_fee);
            // base + delta proportional to net_stake/self.max_net_stake
            self.base_fee + (delta as u128 * net_stake as u128 / self.max_net_stake as u128) as u32
        };

        // apply fee basis_points, 100=1%
        (self.liq_unstake_msol_fees as u128 * share_fee_bp as u128 / 10000) as u64
    }
}

//-----------------------------------------------------
