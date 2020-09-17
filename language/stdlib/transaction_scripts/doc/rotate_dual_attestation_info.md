
<a name="SCRIPT"></a>

# Script `rotate_dual_attestation_info.move`

### Table of Contents

-  [Function `rotate_dual_attestation_info`](#SCRIPT_rotate_dual_attestation_info)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
        -  [Events](#SCRIPT_@Events)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)



<a name="SCRIPT_rotate_dual_attestation_info"></a>

## Function `rotate_dual_attestation_info`


<a name="SCRIPT_@Summary"></a>

### Summary

Updates the url used for off-chain communication, and the public key used to verify dual
attestation on-chain. Transaction can be sent by any account that has dual attestation
information published under it. In practice the only such accounts are Designated Dealers and
Parent VASPs.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

Updates the <code>base_url</code> and <code>compliance_public_key</code> fields of the <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_Credential">DualAttestation::Credential</a></code>
resource published under <code>account</code>. The <code>new_key</code> must be a valid ed25519 public key.


<a name="SCRIPT_@Events"></a>

#### Events

Successful execution of this transaction emits two events:
* A <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_ComplianceKeyRotationEvent">DualAttestation::ComplianceKeyRotationEvent</a></code> containing the new compliance public key, and
the blockchain time at which the key was updated emitted on the <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_Credential">DualAttestation::Credential</a></code>
<code>compliance_key_rotation_events</code> handle published under <code>account</code>; and
* A <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_BaseUrlRotationEvent">DualAttestation::BaseUrlRotationEvent</a></code> containing the new base url to be used for
off-chain communication, and the blockchain time at which the url was updated emitted on the
<code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_Credential">DualAttestation::Credential</a></code> <code>base_url_rotation_events</code> handle published under <code>account</code>.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name      | Type         | Description                                                               |
| ------    | ------       | -------------                                                             |
| <code>account</code> | <code>&signer</code>    | Signer reference of the sending account of the transaction.               |
| <code>new_url</code> | <code>vector&lt;u8&gt;</code> | ASCII-encoded url to be used for off-chain communication with <code>account</code>.  |
| <code>new_key</code> | <code>vector&lt;u8&gt;</code> | New ed25519 public key to be used for on-chain dual attestation checking. |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category             | Error Reason                           | Description                                                                |
| ----------------           | --------------                         | -------------                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_ECREDENTIAL">DualAttestation::ECREDENTIAL</a></code>         | A <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_Credential">DualAttestation::Credential</a></code> resource is not published under <code>account</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_EINVALID_PUBLIC_KEY">DualAttestation::EINVALID_PUBLIC_KEY</a></code> | <code>new_key</code> is not a valid ed25519 public key.                               |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::create_parent_vasp_account</code>
* <code>Script::create_designated_dealer</code>
* <code>Script::rotate_dual_attestation_info</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_dual_attestation_info">rotate_dual_attestation_info</a>(account: &signer, new_url: vector&lt;u8&gt;, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_rotate_dual_attestation_info">rotate_dual_attestation_info</a>(account: &signer, new_url: vector&lt;u8&gt;, new_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_rotate_base_url">DualAttestation::rotate_base_url</a>(account, new_url);
    <a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_rotate_compliance_public_key">DualAttestation::rotate_compliance_public_key</a>(account, new_key)
}
</code></pre>



</details>
