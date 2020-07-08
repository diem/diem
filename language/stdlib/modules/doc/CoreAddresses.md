
<a name="0x1_CoreAddresses"></a>

# Module `0x1::CoreAddresses`

### Table of Contents

-  [Function `LIBRA_ROOT_ADDRESS`](#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS)
-  [Function `CURRENCY_INFO_ADDRESS`](#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS)
-  [Function `TREASURY_COMPLIANCE_ADDRESS`](#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS)
-  [Function `VM_RESERVED_ADDRESS`](#0x1_CoreAddresses_VM_RESERVED_ADDRESS)
-  [Specification](#0x1_CoreAddresses_Specification)



<a name="0x1_CoreAddresses_LIBRA_ROOT_ADDRESS"></a>

## Function `LIBRA_ROOT_ADDRESS`

The address of the libra root account. This account is
created in genesis, and cannot be changed. This address has
ultimate authority over the permissions granted (or removed) from
accounts on-chain.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">LIBRA_ROOT_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">LIBRA_ROOT_ADDRESS</a>(): address {
    0xA550C18
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_CURRENCY_INFO_ADDRESS"></a>

## Function `CURRENCY_INFO_ADDRESS`

The (singleton) address under which the
<code><a href="Libra.md#0x1_Libra_CurrencyInfo">0x1::Libra::CurrencyInfo</a></code> resource for
every registered currency is published. This is the same as the
<code>LIBRA_ROOT_ADDRESS</code> but there is no requirement that it must
be this from an operational viewpoint, so this is why this is separated out.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CURRENCY_INFO_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CURRENCY_INFO_ADDRESS</a>(): address {
    0xA550C18
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS"></a>

## Function `TREASURY_COMPLIANCE_ADDRESS`

The account address of the treasury and compliance account in
charge of minting/burning and other day-to-day but privileged
operations. The account at this address is created in genesis.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">TREASURY_COMPLIANCE_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">TREASURY_COMPLIANCE_ADDRESS</a>(): address {
    0xB1E55ED
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_VM_RESERVED_ADDRESS"></a>

## Function `VM_RESERVED_ADDRESS`

The reserved address for transactions inserted by the VM into blocks (e.g.
block metadata transactions). Because the transaction is sent from
the VM, an account _cannot_ exist at the
<code>0x0</code> address since there
is no signer for the transaction.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_VM_RESERVED_ADDRESS">VM_RESERVED_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_VM_RESERVED_ADDRESS">VM_RESERVED_ADDRESS</a>(): address {
    0x0
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_Specification"></a>

## Specification

Specification version of
<code><a href="#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">Self::LIBRA_ROOT_ADDRESS</a></code>.


<a name="0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS"></a>


<pre><code><b>define</b> <a href="#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">SPEC_LIBRA_ROOT_ADDRESS</a>(): address {
    0xA550C18
}
</code></pre>


Specification version of
<code><a href="#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">Self::CURRENCY_INFO_ADDRESS</a></code>.


<a name="0x1_CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS"></a>


<pre><code><b>define</b> <a href="#0x1_CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS">SPEC_CURRENCY_INFO_ADDRESS</a>(): address {
    0xA550C18
}
</code></pre>


Specification version of
<code><a href="#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">Self::TREASURY_COMPLIANCE_ADDRESS</a></code>.


<a name="0x1_CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS"></a>


<pre><code><b>define</b> <a href="#0x1_CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS">SPEC_TREASURY_COMPLIANCE_ADDRESS</a>(): address {
    0xB1E55ED
}
</code></pre>


Specification version of
<code><a href="#0x1_CoreAddresses_VM_RESERVED_ADDRESS">Self::VM_RESERVED_ADDRESS</a></code>.


<a name="0x1_CoreAddresses_SPEC_VM_RESERVED_ADDRESS"></a>


<pre><code><b>define</b> <a href="#0x1_CoreAddresses_SPEC_VM_RESERVED_ADDRESS">SPEC_VM_RESERVED_ADDRESS</a>(): address {
    0x0
}
</code></pre>
