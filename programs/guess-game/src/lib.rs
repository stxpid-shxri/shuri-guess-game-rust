use anchor_lang::{
    prelude::*,
    solana_program::{hash::hash, program::invoke, system_instruction::transfer},
};

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("CKWtwTziPzvp7VAVi8tbbBurWbSaG4Fx5icGdbs2n5ck");

#[program]
mod guess_number {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, amount: u64) -> Result<()> {
        let master = &mut ctx.accounts.master;
        let user = &ctx.accounts.user;
        invoke(
            &transfer(&user.key(), &master.key(), amount),
            &[
                user.to_account_info(),
                master.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        master.balance += amount;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let master = &mut ctx.accounts.master;
        let user = &ctx.accounts.user;
        invoke(
            &transfer(&user.key(), &master.key(), amount),
            &[
                user.to_account_info(),
                master.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        master.balance += amount;
        Ok(())
    }

    pub fn create(ctx: Context<Create>, random_number: u64) -> Result<()> {
        let game = &mut ctx.accounts.game;
        game.random_number = random_number;
        game.owner = *ctx.accounts.user.key;
        game.is_win = false;
        Ok(())
    }

    pub fn play(ctx: Context<Play>, selected_number: u64, bet: u64) -> Result<()> {
        let master = &mut ctx.accounts.master;
        let game = &mut ctx.accounts.game;
        let user = &ctx.accounts.user;

        if game.owner != user.key() {
            msg!("Different id");
            return err!(PlayError::Mistmatch);
        }

        if game.is_win {
            return err!(PlayError::WinnerAlready);
        }

        // if mismatch number user will transfer to the master.
        if game.random_number != selected_number {
            invoke(
                &transfer(&user.key(), &master.key(), bet),
                &[
                    user.to_account_info(),
                    master.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;

            master.balance += bet;

            return Ok(());
        }

        **master.to_account_info().try_borrow_mut_lamports()? -= bet;
        **user.to_account_info().try_borrow_mut_lamports()? += bet;
        game.is_win = true;
        master.balance -= bet;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8, seeds=[b"master"], bump)]
    pub master: Account<'info, Master>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Create<'info> {
    #[account(init, payer = user, space = 41 + 8, seeds=[b"game", user.key().as_ref()], bump)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub master: Account<'info, Master>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Play<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub master: Account<'info, Master>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Master {
    pub balance: u64,
}

#[account]
pub struct Game {
    pub random_number: u64,
    pub owner: Pubkey,
    pub is_win: bool,
}

#[error_code]
pub enum PlayError {
    #[msg("Winner already")]
    WinnerAlready,
    #[msg("Mismatch")]
    Mistmatch,
}
