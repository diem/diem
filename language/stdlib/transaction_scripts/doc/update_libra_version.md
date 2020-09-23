
<a name="update_libra_version"></a>

# Script `update_libra_version`



-  [Summary](#@Summary_0)
-  [Technical Description](#@Technical_Description_1)
-  [Parameters](#@Parameters_2)
-  [Common Abort Conditions](#@Common_Abort_Conditions_3)


<a name="@Summary_0"></a>

## Summary

Updates the Libra major version that is stored on-chain and is used by the VM.  This
transaction can only be sent from the Libra Root account.


<a name="@Technical_Description_1"></a>

## Technical Description

Updates the <code><a href="../../modules/doc/LibraVersion.md#0x1_LibraVersion">LibraVersion</a></code> on-chain config and emits a <code><a href="../../modules/doc/LibraConfig.md#0x1_LibraConfig_NewEpochEvent">LibraConfig::NewEpochEvent</a></code> to trigger
a reconfiguration of the system. The <code>major</code> version that is passed in must be strictly greater
than the current major version held on-chain. The VM reads this information and can use it to
preserve backwards compatibility with previous major versions of the VM.


<a name="@Parameters_2"></a>

## Parameters

| Name            | Type      | Description                                                                |
| ------          | ------    | -------------                                                              |
| <code>account</code>       | <code>&signer</code> | Signer reference of the sending account. Must be the Libra Root account.   |
| <code>sliding_nonce</code> | <code>u64</code>     | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction. |
| <code>major</code>         | <code>u64</code>     | The <code>major</code> version of the VM to be used from this transaction on.         |


<a name="@Common_Abort_Conditions_3"></a>

## Common Abort Conditions

| Error Category             | Error Reason                                  | Description                                                                                |
| ----------------           | --------------                                | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>                | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>                | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>       | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ELIBRA_ROOT">CoreAddresses::ELIBRA_ROOT</a></code>                  | <code>account</code> is not the Libra Root account.                                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraVersion.md#0x1_LibraVersion_EINVALID_MAJOR_VERSION_NUMBER">LibraVersion::EINVALID_MAJOR_VERSION_NUMBER</a></code> | <code>major</code> is less-than or equal to the current major version stored on-chain.                |


<pre><code><b>public</b> <b>fun</b> <a href="update_libra_version.md#update_libra_version">update_libra_version</a>(account: &signer, sliding_nonce: u64, major: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="update_libra_version.md#update_libra_version">update_libra_version</a>(account: &signer, sliding_nonce: u64, major: u64) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(account, sliding_nonce);
    <a href="../../modules/doc/LibraVersion.md#0x1_LibraVersion_set">LibraVersion::set</a>(account, major)
}
</code></pre>



</details>
