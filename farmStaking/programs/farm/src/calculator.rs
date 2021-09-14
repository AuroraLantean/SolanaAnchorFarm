//! Utility functions for calculating unlock schedules for a vesting account.
use anchor_lang::prelude::msg;

//use crate::Vesting;//struct_name

pub fn calculate_reward(stake_amount: u64, total_pool_deposit: u64, alloc_point: u16, total_alloc_point: u16, reward_rate: u64, time_now: u32, stake_prev: u32) -> u64 {
  msg!("--------------== calculate_reward");

  msg!("stake_amount = {}, total_pool_deposit = {}", stake_amount, total_pool_deposit);

  let alloc_point_ratio = alloc_point.checked_div(total_alloc_point).expect("eCal2");//u16
  msg!("alloc_point = {}, total_alloc_point = {}, alloc_point_ratio = {}", alloc_point, total_alloc_point, alloc_point_ratio);

  let staking_period = time_now.checked_sub(stake_prev).expect("eCal1");
  msg!("stake_prev = {}, deposit_now = {}, staking_period = {}", stake_prev, time_now, staking_period);
  
  //user reward = user deposit/total deposit in the pool * allocationPoint/Total allocationPoint from all Pools * # of Aries per sec*staking peroid in seconds
  let mut reward_debt = 0;
  if total_pool_deposit > 0 {
    reward_debt = stake_amount.checked_div(total_pool_deposit).expect("eCal3").checked_mul(u64::from(alloc_point_ratio)).expect("eCal4").checked_mul(reward_rate).expect("eCal5").checked_mul(u64::from(staking_period)).expect("eCal6");
  }
  msg!("reward_debt: {}", reward_debt);
  reward_debt
}
/*
pub fn available_for_withdrawal(vesting: &Vesting, current_ts: i64) -> u64 {
    std::cmp::min(outstanding_vested(vesting, current_ts), balance(vesting))
}

// The amount of funds currently in the vault.
fn balance(vesting: &Vesting) -> u64 {
    vesting
        .outstanding
        .checked_sub(vesting.whitelist_owned)
        .unwrap()
}

// The amount of outstanding locked tokens vested. Note that these
// tokens might have been transferred to whitelisted programs.
fn outstanding_vested(vesting: &Vesting, current_ts: i64) -> u64 {
    total_vested(vesting, current_ts)
        .checked_sub(withdrawn_amount(vesting))
        .unwrap()
}

// Returns the amount withdrawn from this vesting account.
fn withdrawn_amount(vesting: &Vesting) -> u64 {
    vesting
        .start_balance
        .checked_sub(vesting.outstanding)
        .unwrap()
}

// Returns the total vested amount up to the given ts, assuming zero
// withdrawals and zero funds sent to other programs.
fn total_vested(vesting: &Vesting, current_ts: i64) -> u64 {
    if current_ts < vesting.start_ts {
        0
    } else if current_ts >= vesting.end_ts {
        vesting.start_balance
    } else {
        linear_unlock(vesting, current_ts).unwrap()
    }
}

fn linear_unlock(vesting: &Vesting, current_ts: i64) -> Option<u64> {
    // Signed division not supported.
    let current_ts = current_ts as u64;
    let start_ts = vesting.start_ts as u64;
    let end_ts = vesting.end_ts as u64;

    // If we can't perfectly partition the vesting window,
    // push the start of the window back so that we can.
    //
    // This has the effect of making the first vesting period shorter
    // than the rest.
    let shifted_start_ts =
        start_ts.checked_sub(end_ts.checked_sub(start_ts)? % vesting.period_count)?;

    // Similarly, if we can't perfectly divide up the vesting rewards
    // then make the first period act as a cliff, earning slightly more than
    // subsequent periods.
    let reward_overflow = vesting.start_balance % vesting.period_count;

    // Reward per period ignoring the overflow.
    let reward_per_period =
        (vesting.start_balance.checked_sub(reward_overflow)?).checked_div(vesting.period_count)?;

    // Number of vesting periods that have passed.
    let current_period = {
        let period_secs =
            (end_ts.checked_sub(shifted_start_ts)?).checked_div(vesting.period_count)?;
        let current_period_count =
            (current_ts.checked_sub(shifted_start_ts)?).checked_div(period_secs)?;
        std::cmp::min(current_period_count, vesting.period_count)
    };

    if current_period == 0 {
        return Some(0);
    }

    current_period
        .checked_mul(reward_per_period)?
        .checked_add(reward_overflow)
}
*/