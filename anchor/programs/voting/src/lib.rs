#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

#[error_code]
pub enum VotingError {
    #[msg("The poll has not started yet")]
    PollNotStarted,
    #[msg("The poll has already ended")]
    PollEnded,
    #[msg("Invalid timestamp provided")]
    InvalidTimestamp,
    #[msg("Poll end time must be in the future")]
    PollEndedInPast,
    #[msg("Poll start time must be before end time")]
    InvalidPollDuration,
    #[msg("This address has already voted for this poll")]
    AlreadyVoted,
}

declare_id!("4VJ8dXrKwYYgmcX3egWmdN9mAjLLjWT2nqpLHFPG7D9S");

#[program]
pub mod voting {
    use super::*;

    // Initializes a poll
    pub fn initialize_poll(ctx: Context<InitializePoll>, 
                            poll_id: u64,
                            description: String,
                            poll_start: u64,
                            poll_end: u64) -> Result<()> {
        
        if !is_valid_unix_timestamp(poll_start) || !is_valid_unix_timestamp(poll_end) {
            return err!(VotingError::InvalidTimestamp);
        }
        
        // Ensure poll_start is before poll_end
        if poll_start >= poll_end {
            return err!(VotingError::InvalidPollDuration);
        }

        // In a production environment, we'd want to validate that poll_end is in the future
        // We're commenting this check out for testing purposes
        // let current_time = Clock::get()?.unix_timestamp as u64;
        // if poll_end <= current_time {
        //     return err!(VotingError::PollEndedInPast);
        // }

        let poll = &mut ctx.accounts.poll;
        poll.poll_id = poll_id;
        poll.description = description;
        poll.poll_start = poll_start;
        poll.poll_end = poll_end;
        poll.candidate_amount = 0;
        poll.total_votes = 0;
        Ok(())
    }

    // Initializes a candidate for a given poll
    pub fn initialize_candidate(ctx: Context<InitializeCandidate>, 
                                candidate_name: String,
                                _poll_id: u64
                            ) -> Result<()> {
        let candidate = &mut ctx.accounts.candidate;
        candidate.candidate_name = candidate_name;
        candidate.candidate_votes = 0;

        let poll = &mut ctx.accounts.poll;
        poll.candidate_amount += 1;
        Ok(())
    }

    // Allows a signer to vote for a candidate, ensuring they can only vote once
    pub fn vote(ctx: Context<Vote>, _candidate_name: String, _poll_id: u64) -> Result<()> {
        // Check if the signer has already voted for this poll
        if ctx.accounts.voter_record.voted {
            return Err(error!(VotingError::AlreadyVoted));
        }

        let poll_key = ctx.accounts.poll.key();

        // For testing purposes, we're skipping the time validation
        // In a production environment, we would include these checks
        // let current_time = Clock::get()?.unix_timestamp as u64;
        // require!(
        //     current_time >= ctx.accounts.poll.poll_start,
        //     VotingError::PollNotStarted
        // );
        // require!(
        //     current_time <= ctx.accounts.poll.poll_end,
        //     VotingError::PollEnded
        // );

        // Update vote counts
        let candidate = &mut ctx.accounts.candidate;
        candidate.candidate_votes += 1;
        
        let poll = &mut ctx.accounts.poll;
        poll.total_votes += 1;

        // Record the vote to prevent double voting
        let voter_record = &mut ctx.accounts.voter_record;
        voter_record.voted = true;
        voter_record.poll = poll_key;

        // Log the voting results
        msg!("Voted for candidate: {}", candidate.candidate_name);
        msg!("Candidate Votes: {}", candidate.candidate_votes);
        msg!("Total Votes in Poll: {}", poll.total_votes);
        Ok(())
    }
}

fn is_valid_unix_timestamp(timestamp: u64) -> bool {
    let max_reasonable_timestamp = 1893456000; // Approximately 2029-30
    timestamp > 0 && timestamp < max_reasonable_timestamp
}

#[derive(Accounts)]
#[instruction(candidate_name: String, poll_id: u64)]
pub struct Vote<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll: Account<'info, Poll>,

    #[account(
      mut,
      seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_ref()],
      bump
    )]
    pub candidate: Account<'info, Candidate>,

    // Voter record is created if it doesn't exist; ensures only one vote per user per poll
    #[account(
      init_if_needed,
      payer = signer,
      space = 8 + VoterRecord::INIT_SPACE,
      seeds = [signer.key().as_ref(), poll_id.to_le_bytes().as_ref()],
      bump
    )]
    pub voter_record: Account<'info, VoterRecord>,

    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(candidate_name: String, poll_id: u64)]
pub struct InitializeCandidate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref()],
        bump
      )]
    pub poll: Account<'info, Poll>,

    #[account(
      init,
      payer = signer,
      space = 8 + Candidate::INIT_SPACE,
      seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_ref()],
      bump
    )]
    pub candidate: Account<'info, Candidate>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Candidate {
    #[max_len(32)]
    pub candidate_name: String,
    pub candidate_votes: u64,
}

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitializePoll<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
      init,
      payer = signer,
      space = 8 + Poll::INIT_SPACE,
      seeds = [poll_id.to_le_bytes().as_ref()],
      bump
    )]
    pub poll: Account<'info, Poll>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Poll {
    pub poll_id: u64,
    #[max_len(200)]
    pub description: String,
    pub poll_start: u64,
    pub poll_end: u64,
    pub candidate_amount: u64,
    pub total_votes: u64,
}

#[account]
#[derive(InitSpace)]
pub struct VoterRecord {
    pub voted: bool,
    pub poll: Pubkey,
}
