mod account;
mod signature;

pub use account::*;
pub use signature::*;

pub fn all_natives<N>() -> Vec<(&'static str, &'static str, N)>
where
    N: From<NativeAccountCreateSigner>
        + From<NativeAccountDestroySigner>
        + From<NativeSignatureEd25519ValidatePubkey>
        + From<NativeSignatureEd25519Verify>,
{
    vec![
        (
            "DiemAccount",
            "create_signer",
            NativeAccountCreateSigner.into(),
        ),
        (
            "DiemAccount",
            "destroy_signer",
            NativeAccountDestroySigner.into(),
        ),
        (
            "Signature",
            "ed25519_validate_pubkey",
            NativeSignatureEd25519ValidatePubkey.into(),
        ),
        (
            "Signature",
            "ed25519_verify",
            NativeSignatureEd25519Verify.into(),
        ),
    ]
}
