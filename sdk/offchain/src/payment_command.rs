// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::types::{
    Command, CommandRequestObject, PaymentActorObject, PaymentCommandObject, PaymentObject, Status,
};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Origin {
    Inbound,
    Outbound,
}

impl Origin {
    pub fn is_inbound(&self) -> bool {
        matches!(self, Origin::Inbound)
    }

    pub fn is_outbound(&self) -> bool {
        matches!(self, Origin::Outbound)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Actor {
    Sender,
    Receiver,
}

impl Actor {
    pub fn counterparty_actor(&self) -> Self {
        match self {
            Actor::Sender => Actor::Receiver,
            Actor::Receiver => Actor::Sender,
        }
    }
}

#[derive(Debug)]
pub struct PaymentCommand {
    payment: PaymentObject,
    payment_state: PaymentState,
    origin: Origin,
    my_actor: Actor,
    cid: Uuid,
}

impl PaymentCommand {
    pub fn new(
        payment: PaymentObject,
        origin: Origin,
        my_actor: Actor,
        cid: Uuid,
        prior: Option<&Self>,
    ) -> Result<Self, &'static str> {
        let payment_state = Self::validate(origin, my_actor, &payment, prior)?;

        Ok(Self {
            payment,
            payment_state,
            origin,
            my_actor,
            cid,
        })
    }

    pub fn payment(&self) -> &PaymentObject {
        &self.payment
    }

    pub fn origin(&self) -> Origin {
        self.origin
    }

    pub fn cid(&self) -> Uuid {
        self.cid
    }

    pub fn reference_id(&self) -> Uuid {
        self.payment().reference_id()
    }

    pub fn my_actor(&self) -> Actor {
        self.my_actor
    }

    pub fn counterparty_actor(&self) -> Actor {
        self.my_actor.counterparty_actor()
    }

    pub fn counterparty_actor_object(&self) -> &PaymentActorObject {
        self.payment
            .actor_object_by_actor(self.counterparty_actor())
    }

    pub fn my_actor_object(&self) -> &PaymentActorObject {
        self.payment.actor_object_by_actor(self.my_actor())
    }

    pub fn payment_state(&self) -> PaymentState {
        self.payment_state
    }

    pub fn follow_up_action(&self) -> Option<PaymentAction> {
        PaymentAction::follow_up_action(self.payment_state(), self.my_actor())
    }

    pub fn to_request(&self) -> CommandRequestObject {
        let command = Command::PaymentCommand(PaymentCommandObject::new(self.payment.clone()));
        CommandRequestObject::new(command, self.cid())
    }

    fn validate(
        origin: Origin,
        my_actor: Actor,
        payment_object: &PaymentObject,
        prior: Option<&PaymentCommand>,
    ) -> Result<PaymentState, &'static str> {
        let payment_state =
            PaymentState::from_payment(payment_object).ok_or("invalid payment state")?;
        // Validate state trigger actor
        if origin.is_inbound() && my_actor.counterparty_actor() != payment_state.trigger_actor() {
            return Err("Should not produce");
        }

        if let Some(prior) = prior {
            // Does the prior command have the same reference_id?
            if payment_object.reference_id() != prior.payment().reference_id() {
                return Err("payment object reference_id does not match");
            }

            // Validate actor object
            if origin.is_inbound()
                && payment_object.actor_object_by_actor(my_actor) != prior.my_actor_object()
            {
                return Err("invalid overwrite");
            }

            // Validate WriteOnce fields
            payment_object
                .validate_write_once_fields(prior.payment())
                .map_err(|_| "write once fields error")?;

            // Validate transition
            if !PaymentState::is_valid_transition(prior.payment_state(), payment_state) {
                return Err("invalid state transition");
            }
        } else {
            // Must be an initial Command
            if !matches!(payment_state, PaymentState::SenderInit) {
                return Err("must be initial or unable to find prior payment object");
            }
        }

        Ok(payment_state)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaymentState {
    // S_INIT
    SenderInit,
    // S_ABORT
    SenderAbort,
    // S_SOFT
    SenderSoft,
    // S_SOFT_SEND
    SenderSoftSend,
    // READY
    Ready,
    // R_ABORT
    RecieverAbort,
    // R_SOFT
    RecieverSoft,
    // R_SOFT_SEND
    RecieverSoftSend,
    // R_SEND
    RecieverSend,
}

impl PaymentState {
    pub fn is_valid_transition(from: Self, to: Self) -> bool {
        use PaymentState::*;

        match from {
            SenderInit => matches!(to, RecieverAbort | RecieverSoft | RecieverSend),
            SenderSoft => matches!(to, RecieverAbort | RecieverSoftSend),
            SenderSoftSend => matches!(to, RecieverAbort | RecieverSend),
            RecieverSoft => matches!(to, SenderAbort | SenderSoftSend),
            RecieverSoftSend => matches!(to, SenderAbort | Ready),
            RecieverSend => matches!(to, SenderAbort | SenderSoft | Ready),
            RecieverAbort | SenderAbort | Ready => false,
        }
    }

    pub fn trigger_actor(&self) -> Actor {
        match self {
            PaymentState::SenderInit
            | PaymentState::SenderAbort
            | PaymentState::SenderSoft
            | PaymentState::SenderSoftSend
            | PaymentState::Ready => Actor::Sender,

            PaymentState::RecieverAbort
            | PaymentState::RecieverSoft
            | PaymentState::RecieverSoftSend
            | PaymentState::RecieverSend => Actor::Receiver,
        }
    }

    pub fn from_payment(payment: &PaymentObject) -> Option<Self> {
        let state = match (
            payment.sender().status().status(),
            payment.sender().additional_kyc_data(),
            payment.receiver().status().status(),
            payment.receiver().additional_kyc_data(),
        ) {
            // Basic Kyc exchange flow
            (Status::NeedsKycData, None, Status::None, None) => {
                #[allow(clippy::question_mark)]
                if payment.sender().kyc_data().is_none() {
                    return None;
                }

                PaymentState::SenderInit
            }
            (Status::NeedsKycData, _, Status::ReadyForSettlement, None) => {
                if payment.receiver().kyc_data().is_none()
                    || payment.recipient_signature().is_none()
                {
                    return None;
                }

                PaymentState::RecieverSend
            }
            // https://dip.diem.com/dip-1/#sinit---rabort-step-9 states that the abort code needs
            // to be one of "no-kyc-needed" or "rejected" but "no-kyc-needed" isn't a valid abort
            // code
            (Status::NeedsKycData, _, Status::Abort, _) => PaymentState::RecieverAbort,
            (Status::Abort, _, Status::ReadyForSettlement, _) => PaymentState::SenderAbort,
            (Status::ReadyForSettlement, _, Status::ReadyForSettlement, _) => PaymentState::Ready,

            // Soft-match disambiguation states
            (Status::NeedsKycData, None, Status::SoftMatch, None) => PaymentState::RecieverSoft,
            (Status::NeedsKycData, Some(_), Status::SoftMatch, None) => {
                PaymentState::SenderSoftSend
            }
            (Status::SoftMatch, _, Status::ReadyForSettlement, None) => PaymentState::SenderSoft,
            (Status::SoftMatch, _, Status::ReadyForSettlement, Some(_)) => {
                PaymentState::RecieverSoftSend
            }

            _ => return None,
        };

        Some(state)
    }
}

pub enum PaymentAction {
    EvaluateKycData,
    ReviewKycData,
    ClearSoftMatch,
    SubmitTransaction,
}

impl PaymentAction {
    pub fn follow_up_action(payment_state: PaymentState, actor: Actor) -> Option<PaymentAction> {
        let action = match (payment_state, actor) {
            (PaymentState::SenderInit, Actor::Receiver) => PaymentAction::EvaluateKycData,
            (PaymentState::SenderSoft, Actor::Receiver) => PaymentAction::ClearSoftMatch,
            (PaymentState::SenderSoftSend, Actor::Receiver) => PaymentAction::ReviewKycData,
            (PaymentState::Ready, Actor::Sender) => PaymentAction::SubmitTransaction,
            (PaymentState::RecieverSend, Actor::Sender) => PaymentAction::EvaluateKycData,
            (PaymentState::RecieverSoft, Actor::Sender) => PaymentAction::ClearSoftMatch,
            (PaymentState::RecieverSoftSend, Actor::Sender) => PaymentAction::ReviewKycData,
            _ => return None,
        };

        Some(action)
    }
}
