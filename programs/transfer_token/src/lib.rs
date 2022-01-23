use anchor_lang::prelude::*;
use anchor_spl::{self, associated_token::{AssociatedToken}, token::{self, Mint, TokenAccount, Token, Transfer}};
use anchor_lang::solana_program::system_program;


declare_id!("CyW3dajTAaoH4oh2q75DbX7PcvN92dR6JiMVeVQc783Y");

#[program]
pub mod sship {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, start: u64) -> ProgramResult {
        let counter = &mut ctx.accounts.counter_ship;
        counter.count = start;
        Ok(())
    }

    pub fn create_ship(ctx: Context<CreateShip>, amount_ship: u32, detail: u32) -> ProgramResult{
        let ship_detail: &mut Account<ShipDetail> = &mut ctx.accounts.ship_detail;
        let player: &Signer = &ctx.accounts.player;
        ship_detail.player = *player.key;
        ship_detail.detail = detail;
        let counter_ship = &mut ctx.accounts.counter_ship;
        counter_ship.count += 1;
        Ok(())
    }

    pub fn create_mint_and_vault(ctx: Context<MintAndVault>, _decimals: u8, amount: u64) -> ProgramResult {        
        let mint_to_ctx = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info()
        };
        return token::mint_to(CpiContext::new(ctx.accounts.token_program.to_account_info(), mint_to_ctx), amount);

    }

    pub fn transfer_wrapper(ctx: Context<TransferWrapper>, amount: u64) -> ProgramResult {
        msg!("starting tokens: {}", ctx.accounts.sender_token.amount);
        token::transfer(ctx.accounts.transfer_ctx(), amount)?;
        ctx.accounts.sender_token.reload()?;
        msg!("remaining tokens: {}", ctx.accounts.sender_token.amount);
        Ok(())
    }
}


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 48)]
    pub counter_ship: Account<'info, CounterShip>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateShip<'info>{
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(init, payer = player, space=8+32+32+32)]
    pub ship_detail: Account<'info, ShipDetail>,
    #[account(mut)]
    pub counter_ship: Account<'info, CounterShip>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>
}

#[account]
pub struct CounterShip {
    pub count: u64,
}

#[account]
pub struct ShipDetail{
    pub player: Pubkey,
    pub id: u32,
    pub detail: u32
}

#[derive(Accounts)]
#[instruction(_decimals: u8)]
pub struct MintAndVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(init, payer = authority, mint::decimals = _decimals, mint::authority = authority, mint::freeze_authority = authority)]
    pub mint: Account<'info, Mint>,

    #[account(init, payer = authority, associated_token::mint = mint, associated_token::authority = authority)]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct TransferWrapper<'info> {
    pub sender: Signer<'info>,
    #[account(mut)]
    pub sender_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub receiver_token: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

impl<'info> TransferWrapper<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.sender_token.to_account_info(),
                to: self.receiver_token.to_account_info(),
                authority: self.sender.to_account_info(),
            },
        )
    }
}