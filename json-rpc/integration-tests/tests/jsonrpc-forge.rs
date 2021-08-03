// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use forge::{forge_main, ForgeConfig, LocalFactory, Options, Result};
use jsonrpc_integration_tests::*;

fn main() -> Result<()> {
    let tests = ForgeConfig::default()
        .with_public_usage_tests(&[
            &CurrencyInfo,
            &BlockMetadata,
            &OldMetadata,
            &AccoutNotFound,
            &UnknownAccountRoleType,
            &DesignatedDealerPreburns,
            &ParentVaspAccountRole,
            &GetAccountByVersion,
            &ChildVaspAccountRole,
            &PeerToPeerWithEvents,
            &PeerToPeerErrorExplination,
            &ReSubmittingTransactionWontFail,
            &MempoolValidationError,
            &ExpiredTransaction,
            &RotateComplianceKeyEvent,
            &CreateAccountEvent,
            &GetTransactionsWithoutEvents,
            &GetAccountTransactionsWithoutEvents,
            &GetAccountTransactionsWithProofs,
            &GetTransactionsWithProofs,
            &GetTreasuryComplianceAccount,
            &GetEventsWithProofs,
            &GetEventByVersionWithProofTest,
            &GetResourcesTest,
            &MultiAgentPaymentOverDualAttestationLimit,
            &GetAccumulatorConsistencyProof,
            &NoUnknownEvents,
        ])
        .with_admin_tests(&[
            &PreburnAndBurnEvents,
            &CancleBurnEvent,
            &UpdateExchangeRateEvent,
            &MintAndReceivedMintEvents,
            &AddAndRemoveVaspDomain,
            &MultiAgentRotateAuthenticationKeyAdminScript,
            &MultiAgentRotateAuthenticationKeyAdminScriptFunction,
            &UpgradeEventAndNewEpoch,
            &UpgradeDiemVersion,
        ]);

    let options = Options::from_args();
    forge_main(tests, LocalFactory::from_workspace()?, &options)
}
