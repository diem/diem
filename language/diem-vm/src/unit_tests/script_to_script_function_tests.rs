// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ADD_CURRENCY_TO_ACCOUNT_BYTES, ADD_RECOVERY_ROTATION_CAPABILITY_BYTES,
    ADD_VALIDATOR_AND_RECONFIGURE_BYTES, BURN_BYTES, BURN_TXN_FEES_BYTES, CANCEL_BURN_BYTES,
    CREATE_CHILD_VASP_ACCOUNT_BYTES, CREATE_DESIGNATED_DEALER_BYTES,
    CREATE_PARENT_VASP_ACCOUNT_BYTES, CREATE_RECOVERY_ADDRESS_BYTES,
    CREATE_VALIDATOR_ACCOUNT_BYTES, CREATE_VALIDATOR_OPERATOR_ACCOUNT_BYTES, FREEZE_ACCOUNT_BYTES,
    PEER_TO_PEER_WITH_METADATA_BYTES, PREBURN_BYTES, PUBLISH_SHARED_ED25519_PUBLIC_KEY_BYTES,
    REGISTER_VALIDATOR_CONFIG_BYTES, REMOVE_VALIDATOR_AND_RECONFIGURE_BYTES,
    ROTATE_AUTHENTICATION_KEY_BYTES, ROTATE_AUTHENTICATION_KEY_WITH_NONCE_ADMIN_BYTES,
    ROTATE_AUTHENTICATION_KEY_WITH_NONCE_BYTES,
    ROTATE_AUTHENTICATION_KEY_WITH_RECOVERY_ADDRESS_BYTES, ROTATE_DUAL_ATTESTATION_INFO_BYTES,
    ROTATE_SHARED_ED25519_PUBLIC_KEY_BYTES, SET_VALIDATOR_CONFIG_AND_RECONFIGURE_BYTES,
    SET_VALIDATOR_OPERATOR_BYTES, SET_VALIDATOR_OPERATOR_WITH_NONCE_ADMIN_BYTES, TIERED_MINT_BYTES,
    UNFREEZE_ACCOUNT_BYTES, UPDATE_DIEM_VERSION_BYTES, UPDATE_DUAL_ATTESTATION_LIMIT_BYTES,
    UPDATE_EXCHANGE_RATE_BYTES, UPDATE_MINTING_ABILITY_BYTES,
};
use diem_framework_releases::legacy::transaction_scripts::*;

#[test]
fn test_byte_patterns() {
    use LegacyStdlibScript as S;
    for script in LegacyStdlibScript::all() {
        let bytes_from_pattern = match script {
            S::AddCurrencyToAccount => ADD_CURRENCY_TO_ACCOUNT_BYTES!().to_vec(),
            S::AddRecoveryRotationCapability => ADD_RECOVERY_ROTATION_CAPABILITY_BYTES!().to_vec(),
            S::AddValidatorAndReconfigure => ADD_VALIDATOR_AND_RECONFIGURE_BYTES!().to_vec(),
            S::Burn => BURN_BYTES!().to_vec(),
            S::BurnTxnFees => BURN_TXN_FEES_BYTES!().to_vec(),
            S::CancelBurn => CANCEL_BURN_BYTES!().to_vec(),
            S::CreateChildVaspAccount => CREATE_CHILD_VASP_ACCOUNT_BYTES!().to_vec(),
            S::CreateDesignatedDealer => CREATE_DESIGNATED_DEALER_BYTES!().to_vec(),
            S::CreateParentVaspAccount => CREATE_PARENT_VASP_ACCOUNT_BYTES!().to_vec(),
            S::CreateRecoveryAddress => CREATE_RECOVERY_ADDRESS_BYTES!().to_vec(),
            S::CreateValidatorAccount => CREATE_VALIDATOR_ACCOUNT_BYTES!().to_vec(),
            S::CreateValidatorOperatorAccount => {
                CREATE_VALIDATOR_OPERATOR_ACCOUNT_BYTES!().to_vec()
            }
            S::FreezeAccount => FREEZE_ACCOUNT_BYTES!().to_vec(),
            S::PeerToPeerWithMetadata => PEER_TO_PEER_WITH_METADATA_BYTES!().to_vec(),
            S::Preburn => PREBURN_BYTES!().to_vec(),
            S::PublishSharedEd2551PublicKey => PUBLISH_SHARED_ED25519_PUBLIC_KEY_BYTES!().to_vec(),
            S::RegisterValidatorConfig => REGISTER_VALIDATOR_CONFIG_BYTES!().to_vec(),
            S::RemoveValidatorAndReconfigure => REMOVE_VALIDATOR_AND_RECONFIGURE_BYTES!().to_vec(),
            S::RotateAuthenticationKey => ROTATE_AUTHENTICATION_KEY_BYTES!().to_vec(),
            S::RotateAuthenticationKeyWithNonce => {
                ROTATE_AUTHENTICATION_KEY_WITH_NONCE_BYTES!().to_vec()
            }
            S::RotateAuthenticationKeyWithNonceAdmin => {
                ROTATE_AUTHENTICATION_KEY_WITH_NONCE_ADMIN_BYTES!().to_vec()
            }
            S::RotateAuthenticationKeyWithRecoveryAddress => {
                ROTATE_AUTHENTICATION_KEY_WITH_RECOVERY_ADDRESS_BYTES!().to_vec()
            }
            S::RotateDualAttestationInfo => ROTATE_DUAL_ATTESTATION_INFO_BYTES!().to_vec(),
            S::RotateSharedEd2551PublicKey => ROTATE_SHARED_ED25519_PUBLIC_KEY_BYTES!().to_vec(),
            S::SetValidatorConfigAndReconfigure => {
                SET_VALIDATOR_CONFIG_AND_RECONFIGURE_BYTES!().to_vec()
            }
            S::SetValidatorOperator => SET_VALIDATOR_OPERATOR_BYTES!().to_vec(),
            S::SetValidatorOperatorWithNonceAdmin => {
                SET_VALIDATOR_OPERATOR_WITH_NONCE_ADMIN_BYTES!().to_vec()
            }
            S::TieredMint => TIERED_MINT_BYTES!().to_vec(),
            S::UnfreezeAccount => UNFREEZE_ACCOUNT_BYTES!().to_vec(),
            S::UpdateExchangeRate => UPDATE_EXCHANGE_RATE_BYTES!().to_vec(),
            S::UpdateDiemVersion => UPDATE_DIEM_VERSION_BYTES!().to_vec(),
            S::UpdateMintingAbility => UPDATE_MINTING_ABILITY_BYTES!().to_vec(),
            S::UpdateDualAttestationLimit => UPDATE_DUAL_ATTESTATION_LIMIT_BYTES!().to_vec(),
        };
        let bytes_from_file = script.compiled_bytes().into_vec();
        assert_eq!(
            bytes_from_pattern, bytes_from_file,
            "Mismatch for {}",
            script
        );
    }
}
