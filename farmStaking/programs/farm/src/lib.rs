use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, TokenAccount, Transfer, MintTo};
use anchor_lang::solana_program::program_option::COption;
use solana_program::sysvar::clock::Clock;

mod calculator;

declare_id!("EeoH8QGG4krsVjBE16iJg6c8BTx7omDrE5fd3ybsJF3F");

//0:33:27
//pub const LIST_SIZE: usize = 10;
//------------== PoolAcct
#[derive(Accounts)]
pub struct InitPoolAcct<'info> {
  #[account(zero)]
  pool_acct: Loader<'info, PoolAcct>,
  #[account(signer)]
  authority: AccountInfo<'info>,
  //#[account(init, payer = signer, space = 8 + 8)]
  //#[account(init, payer = signer)]
  //rent: Sysvar<'info, Rent>,//due to init
  //clock: Sysvar<'info, Clock>,
}
//=> please minimize large stack variables

#[account(zero_copy)]
pub struct PoolAcct {
  pub authority: Pubkey,
  pub total_alloc_point: u16,//MAX 65535
  pub time_start: u32,
  pub reward_rate: u64,//per second
  pub second_authority: [u8; 32],
  pub pools: [Pool; 20],
}//#D61A1F
#[zero_copy]
pub struct Pool {
    pub token_addr: Pubkey,
    pub total_deposit: u64,
    pub status_array: u8,//0 delete, L1~L13, 14 pending/suspended
    pub alloc_point: u16,
    pub last_reward: u32,
  }
#[derive(Accounts)]
pub struct UpdatePoolAcct<'info> {
  #[account(mut, has_one = authority)] //has_one enforces UpdatePoolAcct.pool_acct.authority == UpdatePoolAcct.authority.key
  pool_acct: Loader<'info, PoolAcct>,
  #[account(signer)]//authority should sign this txn
  authority: AccountInfo<'info>,
}
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct EachPool {
    pub token_addr: Pubkey,
    pub total_deposit: u64,
    pub status_array: u8,
    pub alloc_point: u16,
    pub last_reward: u32,
}

impl From<EachPool> for Pool {
    fn from(e: EachPool) -> Pool {
        Pool {
            token_addr: e.token_addr,
            total_deposit: e.total_deposit,
            status_array: e.status_array,
            alloc_point: e.alloc_point,
            last_reward: e.last_reward,
          }
    }
}

//------------== 
#[program]
pub mod farm {//mod name <- Cargo.toml
  use super::*;
  
  pub fn init_pool_acct(ctx: Context<InitPoolAcct>, time_start: u32, reward_rate: u64) -> ProgramResult {
    msg!("--------------== init_pool_acct");
    let pool_acct = &mut ctx.accounts.pool_acct.load_init()?;
    pool_acct.authority = *ctx.accounts.authority.key;
    //pool_acct.set_second_authority(ctx.accounts.authority.key);
    pool_acct.total_alloc_point = 0;
    pool_acct.time_start = time_start;
    pool_acct.reward_rate = reward_rate;
    msg!("authority = {}", pool_acct.authority);
    //ctx.accounts.clock.unix_timestamp;
    Ok(())
  }
  
  pub fn update_reward_rate(ctx: Context<UpdatePoolAcct>, reward_rate: u64 ) -> ProgramResult {
    msg!("--------------== update_reward_rate");
    let mut pool_acct = ctx.accounts.pool_acct.load_mut()?;
    pool_acct.reward_rate = reward_rate;
    Ok(())
  }
  pub fn update_start_time(ctx: Context<UpdatePoolAcct>, time_start: u32 ) -> ProgramResult {
    msg!("--------------== update_start_time");
    let pool_acct = &mut ctx.accounts.pool_acct.load_mut()?;
    pool_acct.time_start = time_start;
    Ok(())
  }

  pub fn add_pool(ctx: Context<UpdatePoolAcct>, pidu8: u8, alloc_point: u16, with_update: bool, token_addr: Pubkey) -> Result<()> {
    msg!("--------------== add_pool. pid:{}, alloc_point:{}, with_update:{}", pidu8, alloc_point, with_update);
    msg!("token_addr:{}", token_addr);
    let pool_acct = &mut ctx.accounts.pool_acct.load_mut()?;
    let pid = usize::from(pidu8);

    if pool_acct.pools[pid].status_array > 0 {
      msg!("--------== invalid pid");
      //Custom program error: 0x12f
      return Err(ErrorCode::InvaidPid.into());
    }
    if with_update {
      //mass_update_pools();
    }
    let time_now: u32 = Clock::get().expect("err clock").unix_timestamp as u32;
    let last_reward = if time_now > pool_acct.time_start { time_now } else { pool_acct.time_start };

    pool_acct.total_alloc_point = pool_acct.total_alloc_point.checked_add(alloc_point).expect("err total_alloc_point");
    
    pool_acct.pools[pid].status_array = 1;
    pool_acct.pools[pid].alloc_point = alloc_point;
    pool_acct.pools[pid].last_reward = last_reward;
    pool_acct.pools[pid].token_addr = token_addr;
    Ok(())
  }
/*  pub authority: Pubkey,
  pub total_alloc_point: u16,//MAX 65535
  pub time_start: u32,
  pub status_array: [u8; 10],//0 delete, L1~L13, 14 pending/suspended
  pub token_addr: [Pubkey; 10],
  pub alloc_point: [u16; 10],
  pub last_reward: [u32; 10],
*/
  pub fn disable_pool(ctx: Context<UpdatePoolAcct>, pidu8: u8) -> Result<()> {
    msg!("--------------== disable_pool. pid:{}", pidu8);
    let pool_acct = &mut ctx.accounts.pool_acct.load_mut()?;
    let pid = usize::from(pidu8);
    pool_acct.pools[pid].status_array = 204;
    Ok(())
  }
  pub fn set_pool(ctx: Context<UpdatePoolAcct>, pidu8: u8, alloc_point: u16, with_update: bool) -> Result<()> {
    msg!("--------------== set_pool. pid:{}, alloc_point:{}, with_update:{}", pidu8, alloc_point, with_update);
    let pool_acct = &mut ctx.accounts.pool_acct.load_mut()?;
    let pid = usize::from(pidu8);

    if pool_acct.pools[pid].status_array != 204 {
      msg!("--------== invalid pid");
      return Err(ErrorCode::InvaidPid.into());
    }
    if with_update {
      //mass_update_pools();
    }

    pool_acct.total_alloc_point = pool_acct.total_alloc_point.checked_sub(pool_acct.pools[pid].alloc_point).expect("sub").checked_add(alloc_point).expect("add");
    
    pool_acct.pools[pid].status_array = 1;
    pool_acct.pools[pid].alloc_point = alloc_point;

    // pool_acct.pools[pid] = Pool {
    //   shares,
    //   from: *ctx.accounts.authority.key,
    //   //*ctx.accounts.from.key,
    // };
    Ok(())
  }
  //#[access_control(InitializePool::accounts(&ctx, nonce) future_start_time(&ctx, start_ido_ts))]

  // Asserts the IDO starts in the future.
// fn future_start_time<'info>(ctx: &Context<InitializePool<'info>>, start_ido_ts: i64) -> Result<()> {
//   if !(ctx.accounts.clock.unix_timestamp < start_ido_ts) {
//       return Err(ErrorCode::IdoFuture.into());
//   }
//   Ok(())
// }

  //-------------------== Init User PDA #0011ff  
  pub fn init_user_pda(ctx: Context<InitUserPda>, num: u8) -> ProgramResult {
    msg!("--------------== init_user_pda");
    let user_pda = &mut ctx.accounts.user_pda.load_init()?;
    //user_pda.nonce = nonce;
    user_pda.num = num;
    user_pda.user_acct = *ctx.accounts.user_acct.to_account_info().key;
    user_pda.authority = *ctx.accounts.authority.key;
    Ok(())
  }
/*#[derive(Accounts)]
pub struct InitUserPda<'info> {
}*/
  pub fn update_user_pda(ctx: Context<UpdateUserPda>, pidu8: u8, shares: u64, reward_debt: u64) -> ProgramResult {
    msg!("--------------== update_user_pda");
    let pid = usize::from(pidu8);
    let user_pda = &mut ctx.accounts.user_pda.load_mut()?;
    //msg!("shares = {}, reward_debt = {}", user_pda.staking[pid].share, user_pda.staking[pid].reward);
    
    user_pda.staking[pid].share = shares;
    user_pda.staking[pid].reward = reward_debt;
    // user_pda.pools[pid] = Pool {
    //   shares,
    //   from: *ctx.accounts.authority.key,
    //   //*ctx.accounts.from.key,
    // };
    msg!("user_pda: authority= {}", user_pda.authority);
    Ok(())
  }

  //-------------------==
  /*  pub id: u32,
  pub reward_user: Pubkey,
  pub authority: Pubkey, 
  pub lp_acct: [Pubkey; 10],  */
  pub fn make_user_account(ctx: Context<MakeUserAccount>, user_id: u32, reward_user: Pubkey) -> ProgramResult {
    msg!("--------------== make_account");
    let user_acct = &mut ctx.accounts.user_acct.load_init()?;
    user_acct.id = user_id;
    user_acct.reward_user = reward_user;
    user_acct.authority = *ctx.accounts.authority.key;
    Ok(())
  }

  /*pub fn show_reward(ctx: Context<Stake>, pidu8: u8) -> ProgramResult {
    msg!("--------------== record_reward");
    let pid = usize::from(pidu8);
    let user_pda = &mut ctx.accounts.user_pda;
    let pool_acct = &mut ctx.accounts.pool_acct;
    let time_now: u32 = Clock::get().expect("err:clock").unix_timestamp as u32;

    let reward_debt = calculator::calculate_reward( user_pda.staking[pid].amount, pool_acct.total_deposit[pid], pool_acct.pools[pid].alloc_point, pool_acct.total_alloc_point, pool_acct.reward_rate, time_now, user_pda.staking[pid].time);
    msg!("reward_debt: {}", reward_debt);
    user_pda.reward_debt_temp = reward_debt;
    Ok(())
  }*/


  //-----------------== Stake
  //#b000FF
  #[access_control(Stake::ck_signer(&ctx))]
  pub fn stake(ctx: Context<Stake>, amount: u64, pidu8: u8) -> ProgramResult {
    msg!("--------------== stake");
    /*if self.is_entered {
        return  Err(ErrorCode::Reentrancy.into());
      }
      self.is_entered = true;
      self.is_entered = false;
    */
    //-------------==
    // Transfer LP from lp_user to lp_pgid:
    let cpi_accounts = Transfer {
        from: ctx.accounts.lp_user.to_account_info(),
        to: ctx.accounts.lp_pgid.to_account_info(),
        authority: ctx.accounts.authority.clone(),
    };//authority == user pubkey

    let cpi_program= ctx.accounts.token_program.clone();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount).expect("err token::transfer");
    let received = token::accessor::amount(&ctx.accounts.lp_pgid.to_account_info()).expect("err token::accessor");
    msg!("staking lp successful. amount:{}, received:{}", amount, received);

    //-------------==
    let pid = usize::from(pidu8);
    let user_pda = &mut ctx.accounts.user_pda.load_mut()?;
    let pool_acct = &mut ctx.accounts.pool_acct.load_mut()?;
    let time_now: u32 = Clock::get().expect("err:clock").unix_timestamp as u32;

    /**/
    let total_deposit = pool_acct.pools[pid].total_deposit;

    let reward_debt = calculator::calculate_reward( user_pda.staking[pid].amount, total_deposit, pool_acct.pools[pid].alloc_point, pool_acct.total_alloc_point, pool_acct.reward_rate, time_now, user_pda.staking[pid].time);

    user_pda.staking[pid].reward = user_pda.staking[pid].reward.checked_add(reward_debt).expect("err1");
    user_pda.staking[pid].time = time_now;
    user_pda.staking[pid].amount = user_pda.staking[pid].amount.checked_add(amount).expect("err2");
    msg!("time_now = {}", time_now);

    let total_pool_deposit = total_deposit.checked_add(amount).expect("err3");
    pool_acct.pools[pid].total_deposit = total_pool_deposit;
    msg!("accumulated total_pool_deposit = {}", total_pool_deposit);

    //-------------==
    msg!("Mint reward to user account");
    let mint_amount = amount.checked_mul(2).expect("err9");

    //let user_acct = &ctx.accounts.user_acct.load_mut()?;
    //msg!("reward_user: {}", user_acct.reward_user);

    //if a program controls a token account and wants to send/mint tokens to another account, it must sign
    let seeds = &[user_pda.user_acct.as_ref(), &[user_pda.num], ];
    let signer = &[&seeds[..]];

    let cpi_accounts = MintTo {
      mint: ctx.accounts.reward_mint.to_account_info(),
      to: ctx.accounts.reward_user.to_account_info(),
      authority: ctx.accounts.pg_signer.clone()
    };//authority == createTokenAccount's owner arg

    let cpi_program= ctx.accounts.token_program.clone();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::mint_to(cpi_ctx, mint_amount).expect("err mint_to");
    emit!(Deposit {
      sender: *ctx.accounts.authority.key,
      pidu8: 3,
      want_amt: 11,
    });
    Ok(())
  }

  #[access_control(Stake::ck_signer(&ctx))]
  pub fn unstake(ctx: Context<Stake>, amount: u64, pidu8: u8) -> ProgramResult {
    msg!("--------------== unstake");
    // Transfer Reward from reward_user to reward_pgid
    let mint_amount = amount.checked_mul(2).expect("err1");

    let cpi_accounts = Transfer {
      from: ctx.accounts.reward_user.to_account_info(),
      to: ctx.accounts.reward_pgid.to_account_info(),
      authority: ctx.accounts.authority.clone(),
    };//authority == user pubkey

    let cpi_program= ctx.accounts.token_program.clone();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    msg!("check1");
    token::transfer(cpi_ctx, mint_amount).expect("err token::transfer1");
    msg!("transfer1 successful. amount:{}",mint_amount);
    //-------------==

    let pid = usize::from(pidu8);
    let user_pda = &mut ctx.accounts.user_pda.load_mut()?;
    let pool_acct = &mut ctx.accounts.pool_acct.load_mut()?;
    let time_now: u32 = Clock::get().expect("err:clock").unix_timestamp as u32;

    let total_deposit = pool_acct.pools[pid].total_deposit;
    let reward_debt = calculator::calculate_reward( user_pda.staking[pid].amount, total_deposit, pool_acct.pools[pid].alloc_point, pool_acct.total_alloc_point, pool_acct.reward_rate, time_now, user_pda.staking[pid].time);
    msg!("reward_debt: {}", reward_debt);

    user_pda.staking[pid].reward = user_pda.staking[pid].reward.checked_add(reward_debt).expect("err0");
    user_pda.staking[pid].time = time_now;
    user_pda.staking[pid].amount = user_pda.staking[pid].amount.checked_sub(amount).expect("err1");
    msg!("time_now = {}", time_now);

    let total_pool_deposit = total_deposit.checked_sub(amount).expect("err2");
    pool_acct.pools[pid].total_deposit = total_pool_deposit;
    msg!("accumulated total_pool_deposit = {}", total_pool_deposit);

    //-------------==
    msg!("Transfer LP from lp_pgid to usedc_user");
    let seeds = &[user_pda.user_acct.as_ref(), &[user_pda.num], ];
    let signer = &[&seeds[..]];

    msg!("check3. ctx.program_id: {}",ctx.program_id);
    let cpi_accounts = Transfer {
      from: ctx.accounts.lp_pgid.to_account_info(),
      to: ctx.accounts.lp_user.to_account_info(),
      authority: ctx.accounts.pg_signer.clone(),
    };//authority == createTokenAccount's owner arg
    let cpi_program= ctx.accounts.token_program.clone();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    msg!("check4");
    token::transfer(cpi_ctx, amount).expect("token::transfer2");
    msg!("transfer2 successful. amount: {}", amount);
    emit!(Withdraw {
      sender: *ctx.accounts.authority.key,
      pidu8: 5,
      want_amt: 17,
      //label: "hello".to_string(),
    });
    Ok(())
  }

  #[access_control(Stake::ck_signer(&ctx))]
  pub fn send_lp(ctx: Context<Stake>, amount: u64) -> ProgramResult {
    msg!("--------------== send_lp");
    // Transfer Reward from reward_pgid to reward_user
    let user_pda= &ctx.accounts.user_pda.load_mut()?;
    let seeds = &[user_pda.user_acct.as_ref(), &[user_pda.num], ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = Transfer {
      from: ctx.accounts.lp_pgid.to_account_info(),
      to: ctx.accounts.lp_user.to_account_info(),
      authority: ctx.accounts.pg_signer.clone(),
    };//authority == createTokenAccount's owner arg
    msg!("check3");
    let cpi_program= ctx.accounts.token_program.clone();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    msg!("check4");
    token::transfer(cpi_ctx, amount).expect("err token::transfer");
    msg!("transfer successful. amount: {}", amount);
    Ok(())
  }

  #[access_control(Stake::ck_signer(&ctx))]
  pub fn send_reward(ctx: Context<Stake>, amount: u64) -> ProgramResult {
    msg!("--------------== send_reward");
    // Transfer Reward from reward_pgid to reward_user
    let user_pda= &ctx.accounts.user_pda.load_mut()?;
    let seeds = &[user_pda.user_acct.as_ref(), &[user_pda.num], ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = Transfer {
      from: ctx.accounts.reward_pgid.to_account_info(),
      to: ctx.accounts.reward_user.to_account_info(),
      authority: ctx.accounts.pg_signer.clone(),
    };//authority == user pubkey
    msg!("check3");
    let cpi_program= ctx.accounts.token_program.clone();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    msg!("check4");
    token::transfer(cpi_ctx, amount).expect("err token::transfer");
    msg!("transfer successful. amount: {}", amount);
    Ok(())
  }
}
//------------== Context structs
#[derive(Accounts)]
pub struct Stake<'info> {
  // #[account(seeds = [
  //   check.to_account_info().key.as_ref(),
  //   &[check.nonce],
  // ])]
  pg_signer: AccountInfo<'info>,

  #[account(signer)]//authority should sign this txn
  authority: AccountInfo<'info>,

  user_acct: Loader<'info, UserAcct>,

  #[account(mut, "lp_user.owner == *authority.key")]
  lp_user: Account<'info, TokenAccount>,

  #[account(mut, "&lp_pgid.owner == pg_signer.key")]
  lp_pgid: Account<'info, TokenAccount>,
/*#[derive(Accounts)]
pub struct UpdateUserPda<'info> {
  #[account(mut, has_one = authority,
    seeds = [b"anchor", authority.key().as_ref(), user_acct.key().as_ref()],
    bump,)]
  user_pda: Loader<'info, UserPda>,

  #[account(signer)]//authority should sign this txn
  authority: AccountInfo<'info>,
  
  user_acct: Loader<'info, UserAcct>,
}*/
  //, constraint = reward_mint.supply == 0
  #[account(mut,
  "reward_mint.mint_authority == COption::Some(*pg_signer.key)")]
  reward_mint: Account<'info, Mint>,

  #[account(mut, "reward_user.owner == *authority.key")]
  reward_user: Account<'info, TokenAccount>,

  #[account(mut, "&reward_pgid.owner == pg_signer.key")]
  reward_pgid: Account<'info, TokenAccount>,

  //#[account(seeds = [b"anchor", <target>.key().as_ref(), <with-target>.key().as_ref()], bump,)]//checked at making seeds
  #[account(mut)]
  user_pda: Loader<'info, UserPda>,

  #[account(mut)]
  pool_acct: Loader<'info, PoolAcct>,

  // executable, correct ID
  #[account(executable,"token_program.key == &token::ID")]
  token_program: AccountInfo<'info>,
}
  //pgid: AccountInfo<'info>,
  // #[account("from.mint == to.mint")]
  // to: Account<'info, TokenAccount>,

  //#[account(constraint = lp_mint.decimals == redeemable_mint.decimals)]
  //lp_mint: Account<'info, Mint>,

  
//Error processing Instruction 0: Cross-program invocation with unauthorized signer or writable account 
impl<'info> Stake<'info> {
  pub fn ck_signer(ctx: &Context<Stake>) -> Result<()> {
    let user_pda= &ctx.accounts.user_pda.load_mut()?;
    msg!("user_acct:{}", user_pda.user_acct);

    if user_pda.user_acct != *ctx.accounts.authority.to_account_info().key {
      msg!("--------== InvalidUserAcct");
      //custom program error: 0x132
      return Err(ErrorCode::InvalidUserAcct.into());
    }
    let expected_signer = Pubkey::create_program_address(
        &[ctx.accounts.authority.to_account_info().key.as_ref(), &[user_pda.num]],
        ctx.program_id,
    )
    .map_err(|_| ErrorCode::InvalidNonce).expect("map_err");

    if &expected_signer != ctx.accounts.pg_signer.to_account_info().key {
      msg!("--------== InvalidPgSigner");
      //custom program error: 0x131
      return Err(ErrorCode::InvalidPgSigner.into());
    }
    Ok(())
  }
}

//------------== Init User PDA
#[derive(Accounts)] //#009fff 
pub struct InitUserPda<'info> {
  #[account(init,
    seeds = [b"anchor", authority.key().as_ref(), user_acct.key().as_ref()], 
    bump, payer = authority,)]

  user_pda: Loader<'info, UserPda>,

  #[account(signer)]//authority should sign this txn
  authority: AccountInfo<'info>,
  
  user_acct: Loader<'info, UserAcct>,
  rent: Sysvar<'info, Rent>,//for init
  system_program: AccountInfo<'info>,//for init PDA
}


#[account(zero_copy)]
#[derive(Default)]//for PDA
pub struct UserPda {
  pub num: u8,
  pub user_acct: Pubkey,
  pub authority: Pubkey,
  pub staking: [Staking; 20],
  //pub __nonce: u8,
  //pub nonce: u8, somehow already existing!!!
}
#[zero_copy]
#[derive(Default)]//for PDA
pub struct Staking {
  pub amount: u64,
  pub share: u64,
  pub reward: u64,
  pub time: u32,
}
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct EachStaking {
  pub amount: u64,
  pub share: u64,
  pub reward: u64,
  pub time: u32,
}
impl From<EachStaking> for Staking {
    fn from(e: EachStaking) -> Staking {
        Staking {
            amount: e.amount,
            share: e.share,
            reward: e.reward,
            time: e.time,
          }
    }
}
//------------== Update User PDA
#[derive(Accounts)] //#b000FF
pub struct UpdateUserPda<'info> {
  //#[account(mut, associated = authority, with = user_acct)]
  #[account(mut, has_one = authority,
    seeds = [b"anchor", authority.key().as_ref(), user_acct.key().as_ref()],
    bump,)]
  user_pda: Loader<'info, UserPda>,

  #[account(signer)]//authority should sign this txn
  authority: AccountInfo<'info>,
  
  user_acct: Loader<'info, UserAcct>,
}

//------------== 
#[derive(Accounts)]
pub struct MakeUserAccount<'info> {
  #[account(zero)]
  user_acct: Loader<'info, UserAcct>,
  #[account(signer)]//authority should sign this txn
  authority: AccountInfo<'info>, 
  rent: Sysvar<'info, Rent>,//for init
  //system_program: AccountInfo<'info>,//for init PDA
}

#[account(zero_copy)]
pub struct UserAcct {
  pub id: u32,
  pub reward_user: Pubkey,
  pub authority: Pubkey, 
  pub lp_acct: [Pubkey; 10],
}

// impl Default for UserInfo {
//   fn default() -> UserPda { 
//     UserPda{
//       __nonce: 0,
//       first_deposit: 0,
//     }
//   }
// }

//------------== Show Error
#[error]
pub enum ErrorCode {
  #[msg("unauthorized")]
  Unauthorized,
  #[msg("reentrancy")]
  Reentrancy,
  #[msg("state initialized")]
  StateInitialized,
  #[msg("pid not allowed")]
  InvaidPid,
  #[msg("The given nonce does not create a valid program derived address.")]
  InvalidNonce,
  #[msg("PgSigner does not match")]
  InvalidPgSigner,
  #[msg("invalid userAcct")]
  InvalidUserAcct,
}
//------------== Events
#[event] //#FCDE41  
pub struct Deposit {
  #[index]
  pub sender: Pubkey,
  #[index]
  pub pidu8: u8,
  pub want_amt: u64,
}
#[event]
pub struct Withdraw {
  #[index]
  pub sender: Pubkey,
  #[index]
  pub pidu8: u8,
  pub want_amt: u64,
  //#[index]
  //pub label: String,
}
/*
fn not_burned(check: &Check) -> Result<()> {
    if check.burned {
        return Err(ErrorCode::AlreadyBurned.into());
    }
    Ok(())
}
fn only_auth<'info>(ctx: &Context<ExchangeLpForRw<'info>>) -> Result<()> {
  if !(ctx.accounts.pool_account.start_ido_ts < ctx.accounts.clock.unix_timestamp) {
      return Err(ErrorCode::StartIdoTime.into());
  } else if !(ctx.accounts.clock.unix_timestamp < ctx.accounts.pool_account.end_deposits_ts) {
      return Err(ErrorCode::EndDepositsTime.into());
  }
  Ok(())
}*/


  // #[state]
  // pub struct User {
  //   pub authority: Pubkey,
  //   pub pool_array: [UserPool; 10],
  // }
  // #[state]
  // #[derive(Default)]
  // #[derive(Copy)]
  // pub struct UserPool {
  //   pub shares: u64,
  //   pub reward_debt: u64,
  // }

/* 
//--------------------== State variables
#[derive(Accounts)]
pub struct Auth<'info> {
  #[account(signer)]
  authority: AccountInfo<'info>,
}
//#[access_control(Auth::check(&self, &ctx))]
impl<'info> Auth<'info> {
  // Auxiliary account validation requiring program inputs. convention: separate it from the business logic of the instruction handler itself.
  pub fn check(pstate: &Pstate, ctx: &Context<Auth>) -> Result<()> {
    if &pstate.authority != ctx.accounts.authority.key {
      return Err(ErrorCode::Unauthorized.into());
    }
    Ok(())
  }
}
//-------------------==
  #[state] 
  pub struct Pstate {
    pub authority: Pubkey,
    pub is_entered: bool,
    pub num_u64: u64,
    //pub pool_acct: [Pubkey; 10],
    //pub pool_array: [Ppool; 10],
  }
  impl Pstate {//&mut self, 
    pub fn new(ctx: Context<Auth>, num_u64: u64) -> Result<Self> {
      Ok(Self {
        authority: *ctx.accounts.authority.key,
        is_entered : false,
        num_u64: num_u64,
        //pool_array: [
        //   Ppool {
        //   alloc_point: 0,
        // }; 10],
      })
    }//#D6731A

    #[access_control(Auth::check(&self, &ctx))]
    pub fn set_num_u64(&mut self, ctx: Context<Auth>, num_u64: u64) -> Result<()> {
      msg!("--------------== set_num_u64");
      self.num_u64 = num_u64;
      Ok(())
    }

    #[access_control(Auth::check(&self, &ctx))]
    pub fn set_authority(&mut self, ctx: Context<Auth>, authority: Pubkey) -> Result<()> {
      self.authority = authority;
      Ok(())
    }
  }
*/