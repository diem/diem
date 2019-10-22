use std::time::Duration;

use futures::{join, select};
use tokio::prelude::*;
use tokio::timer::delay_for;

use channel::libra_channel::{new_channel, Receiver, Sender, Type, ValidatorMessage};
use libra_types::account_address::AccountAddress;
use libra_types::account_address::ADDRESS_LENGTH;

/// This represents a proposal message from a validator
struct ProposalMsg {
    msg: String,
    validator: AccountAddress,
}

impl ValidatorMessage for ProposalMsg {
    fn get_validator(&self) -> AccountAddress {
        self.validator
    }
}

/// This represents a vote message from a validator
struct VoteMsg {
    msg: String,
    validator: AccountAddress,
}

impl ValidatorMessage for VoteMsg {
    fn get_validator(&self) -> AccountAddress {
        self.validator
    }
}

// Run an infinite loop which receives value from the AsyncRegister and
// "processes" them. Here we are just printing the value
// s.next().await should never return None because poll_next() never returns
// Poll::Ready(None)
async fn recv_loop(mut proposal_msgs: Receiver<ProposalMsg>, mut vote_msgs: Receiver<VoteMsg>) {
    loop {
        select! {
            proposal_msg = proposal_msgs.select_next_some() => { println!("Processed proposal message {:?} from validator {:?}", proposal_msg.msg,proposal_msg.validator) },
            vote_msg = vote_msgs.select_next_some() => { println!("Processed vote message {:?} from validator {:?}", vote_msg.msg,vote_msg.validator) },
            complete => break,
        }
    }
}

// Helper function for demo which sends proposal msgs in a loop
async fn send_proposal_msg_loop(mut sender: Sender<ProposalMsg>) {
    loop {
        sender.put(ProposalMsg {
            msg: "Message".to_string(),
            validator: AccountAddress::new([1u8; ADDRESS_LENGTH]),
        });
        delay_for(Duration::from_secs(2)).await;
    }
}

// Helper function for demo which sends vote msgs in a loop
async fn send_vote_msg_loop(mut sender: Sender<VoteMsg>) {
    loop {
        sender.put(VoteMsg {
            msg: "Message".to_string(),
            validator: AccountAddress::new([0u8; ADDRESS_LENGTH]),
        });
        delay_for(Duration::from_secs(2)).await;
    }
}

#[tokio::main]
async fn main() {
    let (sender, receiver) = new_channel(1, Type::FIFO);
    let (sender2, receiver2) = new_channel(10, Type::FIFO);
    join!(
        recv_loop(receiver, receiver2),
        send_proposal_msg_loop(sender),
        send_vote_msg_loop(sender2),
    );
}
