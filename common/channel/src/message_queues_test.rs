use crate::libra_channel::MessageQueue;
use crate::message_queues::{PerValidatorQueue, QueueStyle, ValidatorMessage};
use libra_types::account_address::AccountAddress;
use libra_types::account_address::ADDRESS_LENGTH;

/// This represents a proposal message from a validator
#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
struct VoteMsg {
    msg: String,
    validator: AccountAddress,
}

impl ValidatorMessage for VoteMsg {
    fn get_validator(&self) -> AccountAddress {
        self.validator
    }
}

#[test]
fn test_fifo() {
    let mut q = PerValidatorQueue::new(QueueStyle::FIFO, 3);
    let validator = AccountAddress::new([0u8; ADDRESS_LENGTH]);

    // Test order
    q.push(ProposalMsg {
        msg: "msg1".to_string(),
        validator,
    });
    q.push(ProposalMsg {
        msg: "msg2".to_string(),
        validator,
    });
    q.push(ProposalMsg {
        msg: "msg3".to_string(),
        validator,
    });
    assert_eq!(q.pop().unwrap().msg, "msg1".to_string());
    assert_eq!(q.pop().unwrap().msg, "msg2".to_string());
    assert_eq!(q.pop().unwrap().msg, "msg3".to_string());

    // Test max queue size
    q.push(ProposalMsg {
        msg: "msg1".to_string(),
        validator,
    });
    q.push(ProposalMsg {
        msg: "msg2".to_string(),
        validator,
    });
    q.push(ProposalMsg {
        msg: "msg3".to_string(),
        validator,
    });
    q.push(ProposalMsg {
        msg: "msg4".to_string(),
        validator,
    });
    assert_eq!(q.pop().unwrap().msg, "msg1".to_string());
    assert_eq!(q.pop().unwrap().msg, "msg2".to_string());
    assert_eq!(q.pop().unwrap().msg, "msg3".to_string());
    assert_eq!(q.pop(), None);
}

#[test]
fn test_lifo() {
    let mut q = PerValidatorQueue::new(QueueStyle::LIFO, 3);
    let validator = AccountAddress::new([0u8; ADDRESS_LENGTH]);

    // Test order
    q.push(ProposalMsg {
        msg: "msg1".to_string(),
        validator,
    });
    q.push(ProposalMsg {
        msg: "msg2".to_string(),
        validator,
    });
    q.push(ProposalMsg {
        msg: "msg3".to_string(),
        validator,
    });
    assert_eq!(q.pop().unwrap().msg, "msg3".to_string());
    assert_eq!(q.pop().unwrap().msg, "msg2".to_string());
    assert_eq!(q.pop().unwrap().msg, "msg1".to_string());

    // Test max queue size
    q.push(ProposalMsg {
        msg: "msg1".to_string(),
        validator,
    });
    q.push(ProposalMsg {
        msg: "msg2".to_string(),
        validator,
    });
    q.push(ProposalMsg {
        msg: "msg3".to_string(),
        validator,
    });
    q.push(ProposalMsg {
        msg: "msg4".to_string(),
        validator,
    });
    assert_eq!(q.pop().unwrap().msg, "msg4".to_string());
    assert_eq!(q.pop().unwrap().msg, "msg3".to_string());
    assert_eq!(q.pop().unwrap().msg, "msg2".to_string());
    assert_eq!(q.pop(), None);
}

#[test]
fn test_fifo_round_robin() {
    let mut q = PerValidatorQueue::new(QueueStyle::FIFO, 3);
    let validator1 = AccountAddress::new([0u8; ADDRESS_LENGTH]);
    let validator2 = AccountAddress::new([1u8; ADDRESS_LENGTH]);
    let validator3 = AccountAddress::new([2u8; ADDRESS_LENGTH]);

    q.push(ProposalMsg {
        msg: "msg1".to_string(),
        validator: validator1,
    });
    q.push(ProposalMsg {
        msg: "msg2".to_string(),
        validator: validator1,
    });
    q.push(ProposalMsg {
        msg: "msg3".to_string(),
        validator: validator1,
    });
    q.push(ProposalMsg {
        msg: "msg1".to_string(),
        validator: validator2,
    });
    q.push(ProposalMsg {
        msg: "msg1".to_string(),
        validator: validator3,
    });

    assert_eq!(
        q.pop().unwrap(),
        ProposalMsg {
            msg: "msg1".to_string(),
            validator: validator1
        }
    );
    assert_eq!(
        q.pop().unwrap(),
        ProposalMsg {
            msg: "msg1".to_string(),
            validator: validator2
        }
    );
    assert_eq!(
        q.pop().unwrap(),
        ProposalMsg {
            msg: "msg1".to_string(),
            validator: validator3
        }
    );
    assert_eq!(
        q.pop().unwrap(),
        ProposalMsg {
            msg: "msg2".to_string(),
            validator: validator1
        }
    );
    assert_eq!(
        q.pop().unwrap(),
        ProposalMsg {
            msg: "msg3".to_string(),
            validator: validator1
        }
    );
    assert_eq!(q.pop(), None);
}

#[test]
fn test_lifo_round_robin() {
    let mut q = PerValidatorQueue::new(QueueStyle::LIFO, 3);
    let validator1 = AccountAddress::new([0u8; ADDRESS_LENGTH]);
    let validator2 = AccountAddress::new([1u8; ADDRESS_LENGTH]);
    let validator3 = AccountAddress::new([2u8; ADDRESS_LENGTH]);

    q.push(ProposalMsg {
        msg: "msg1".to_string(),
        validator: validator1,
    });
    q.push(ProposalMsg {
        msg: "msg2".to_string(),
        validator: validator1,
    });
    q.push(ProposalMsg {
        msg: "msg3".to_string(),
        validator: validator1,
    });
    q.push(ProposalMsg {
        msg: "msg1".to_string(),
        validator: validator2,
    });
    q.push(ProposalMsg {
        msg: "msg1".to_string(),
        validator: validator3,
    });

    assert_eq!(
        q.pop().unwrap(),
        ProposalMsg {
            msg: "msg3".to_string(),
            validator: validator1
        }
    );
    assert_eq!(
        q.pop().unwrap(),
        ProposalMsg {
            msg: "msg1".to_string(),
            validator: validator2
        }
    );
    assert_eq!(
        q.pop().unwrap(),
        ProposalMsg {
            msg: "msg1".to_string(),
            validator: validator3
        }
    );
    assert_eq!(
        q.pop().unwrap(),
        ProposalMsg {
            msg: "msg2".to_string(),
            validator: validator1
        }
    );
    assert_eq!(
        q.pop().unwrap(),
        ProposalMsg {
            msg: "msg1".to_string(),
            validator: validator1
        }
    );
    assert_eq!(q.pop(), None);
}
