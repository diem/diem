
<a name="SCRIPT"></a>

# Script `rotate_dual_attestation_info.move`

### Table of Contents

-  [Function `rotate_dual_attestation_info`](#SCRIPT_rotate_dual_attestation_info)



<a name="SCRIPT_rotate_dual_attestation_info"></a>

## Function `rotate_dual_attestation_info`

Rotate
<code>account</code>'s base URL to
<code>new_url</code> and its compliance public key to
<code>new_key</code>.
Aborts if
<code>account</code> is not a ParentVASP or DesignatedDealer
Aborts if
<code>new_key</code> is not a well-formed public key


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
