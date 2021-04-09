
<a name="0x1_DiemTransactionPublishingOption"></a>

# Module `0x1::DiemTransactionPublishingOption`

This module defines a struct storing the publishing policies for the VM.


-  [Struct `DiemTransactionPublishingOption`](#0x1_DiemTransactionPublishingOption_DiemTransactionPublishingOption)
-  [Resource `HaltAllTransactions`](#0x1_DiemTransactionPublishingOption_HaltAllTransactions)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_DiemTransactionPublishingOption_initialize)
-  [Function `is_script_allowed`](#0x1_DiemTransactionPublishingOption_is_script_allowed)
-  [Function `is_module_allowed`](#0x1_DiemTransactionPublishingOption_is_module_allowed)
-  [Function `set_open_script`](#0x1_DiemTransactionPublishingOption_set_open_script)
-  [Function `set_open_module`](#0x1_DiemTransactionPublishingOption_set_open_module)
-  [Function `halt_all_transactions`](#0x1_DiemTransactionPublishingOption_halt_all_transactions)
-  [Function `resume_transactions`](#0x1_DiemTransactionPublishingOption_resume_transactions)
-  [Function `transactions_halted`](#0x1_DiemTransactionPublishingOption_transactions_halted)
-  [Module Specification](#@Module_Specification_1)
    -  [Initialization](#@Initialization_2)
    -  [Access Control](#@Access_Control_3)
    -  [Helper Functions](#@Helper_Functions_4)


<pre><code><b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="DiemConfig.md#0x1_DiemConfig">0x1::DiemConfig</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_DiemTransactionPublishingOption_DiemTransactionPublishingOption"></a>

## Struct `DiemTransactionPublishingOption`

Defines and holds the publishing policies for the VM. There are three possible configurations:
1. No module publishing, only allow-listed scripts are allowed.
2. No module publishing, custom scripts are allowed.
3. Both module publishing and custom scripts are allowed.
We represent these as the following resource.


<pre><code><b>struct</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>script_allow_list: vector&lt;vector&lt;u8&gt;&gt;</code>
</dt>
<dd>
 Only script hashes in the following list can be executed by the network. If the vector is empty, no
 limitation would be enforced.
</dd>
<dt>
<code>module_publishing_allowed: bool</code>
</dt>
<dd>
 Anyone can publish new module if this flag is set to true.
</dd>
</dl>


</details>

<a name="0x1_DiemTransactionPublishingOption_HaltAllTransactions"></a>

## Resource `HaltAllTransactions`

If published, halts transactions from all accounts except DiemRoot


<pre><code><b>resource</b> <b>struct</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_HaltAllTransactions">HaltAllTransactions</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_DiemTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT"></a>

The script hash already exists in the allowlist


<pre><code><b>const</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT">EALLOWLIST_ALREADY_CONTAINS_SCRIPT</a>: u64 = 1;
</code></pre>



<a name="0x1_DiemTransactionPublishingOption_EHALT_ALL_TRANSACTIONS"></a>

Attempting to publish/unpublish a HaltAllTransactions resource that does not exist.


<pre><code><b>const</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_EHALT_ALL_TRANSACTIONS">EHALT_ALL_TRANSACTIONS</a>: u64 = 2;
</code></pre>



<a name="0x1_DiemTransactionPublishingOption_EINVALID_SCRIPT_HASH"></a>

The script hash has an invalid length


<pre><code><b>const</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_EINVALID_SCRIPT_HASH">EINVALID_SCRIPT_HASH</a>: u64 = 0;
</code></pre>



<a name="0x1_DiemTransactionPublishingOption_SCRIPT_HASH_LENGTH"></a>



<pre><code><b>const</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_SCRIPT_HASH_LENGTH">SCRIPT_HASH_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x1_DiemTransactionPublishingOption_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_initialize">initialize</a>(dr_account: &signer, script_allow_list: vector&lt;vector&lt;u8&gt;&gt;, module_publishing_allowed: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_initialize">initialize</a>(
    dr_account: &signer,
    script_allow_list: vector&lt;vector&lt;u8&gt;&gt;,
    module_publishing_allowed: bool,
) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();
    <a href="Roles.md#0x1_Roles_assert_diem_root">Roles::assert_diem_root</a>(dr_account);

    <a href="DiemConfig.md#0x1_DiemConfig_publish_new_config">DiemConfig::publish_new_config</a>(
        dr_account,
        <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a> {
            script_allow_list, module_publishing_allowed
        }
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the DiemRoot role [[H11]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDiemRoot">Roles::AbortsIfNotDiemRoot</a>{account: dr_account};
<b>include</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp_AbortsIfNotGenesis">DiemTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_PublishNewConfigAbortsIf">DiemConfig::PublishNewConfigAbortsIf</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;;
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_PublishNewConfigEnsures">DiemConfig::PublishNewConfigEnsures</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt; {
    payload: <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a> {
        script_allow_list, module_publishing_allowed
    }};
</code></pre>



</details>

<a name="0x1_DiemTransactionPublishingOption_is_script_allowed"></a>

## Function `is_script_allowed`

Check if sender can execute script with <code>hash</code>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_is_script_allowed">is_script_allowed</a>(account: &signer, hash: &vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_is_script_allowed">is_script_allowed</a>(account: &signer, hash: &vector&lt;u8&gt;): bool {
    // DiemRoot can send any <b>script</b>
    <b>if</b> (<a href="Roles.md#0x1_Roles_has_diem_root_role">Roles::has_diem_root_role</a>(account)) <b>return</b> <b>true</b>;

    // No one <b>except</b> DiemRoot can send scripts when transactions are halted
    <b>if</b> (<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_transactions_halted">transactions_halted</a>()) <b>return</b> <b>false</b>;

    // The adapter passes an empty hash for <b>script</b> functions. All <b>script</b> functions are allowed
    <b>if</b> (<a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(hash)) <b>return</b> <b>true</b>;

    <b>let</b> publish_option = <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();
    // allowlist empty = open publishing, anyone can send txes
    <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&publish_option.script_allow_list)
        // fixed allowlist. check inclusion
        || <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_contains">Vector::contains</a>(&publish_option.script_allow_list, hash)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b>
    !<a href="Roles.md#0x1_Roles_has_diem_root_role">Roles::has_diem_root_role</a>(account) && !<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_transactions_halted">transactions_halted</a>() && !<a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(hash)
    ==&gt; <a href="DiemConfig.md#0x1_DiemConfig_AbortsIfNotPublished">DiemConfig::AbortsIfNotPublished</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;{};
</code></pre>




<a name="0x1_DiemTransactionPublishingOption_AbortsIfNoTransactionPublishingOption"></a>


<pre><code><b>schema</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_AbortsIfNoTransactionPublishingOption">AbortsIfNoTransactionPublishingOption</a> {
    <b>include</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp_is_genesis">DiemTimestamp::is_genesis</a>() ==&gt; <a href="DiemConfig.md#0x1_DiemConfig_AbortsIfNotPublished">DiemConfig::AbortsIfNotPublished</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;{};
}
</code></pre>



</details>

<a name="0x1_DiemTransactionPublishingOption_is_module_allowed"></a>

## Function `is_module_allowed`

Check if a sender can publish a module


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_is_module_allowed">is_module_allowed</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_is_module_allowed">is_module_allowed</a>(account: &signer): bool {
    <b>let</b> publish_option = <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();

    publish_option.module_publishing_allowed || <a href="Roles.md#0x1_Roles_has_diem_root_role">Roles::has_diem_root_role</a>(account)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_AbortsIfNotPublished">DiemConfig::AbortsIfNotPublished</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;{};
</code></pre>



</details>

<a name="0x1_DiemTransactionPublishingOption_set_open_script"></a>

## Function `set_open_script`

Allow the execution of arbitrary script or not.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_set_open_script">set_open_script</a>(dr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_set_open_script">set_open_script</a>(dr_account: &signer) {
    <a href="Roles.md#0x1_Roles_assert_diem_root">Roles::assert_diem_root</a>(dr_account);
    <b>let</b> publish_option = <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();

    publish_option.script_allow_list = <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>();
    <a href="DiemConfig.md#0x1_DiemConfig_set">DiemConfig::set</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;(dr_account, publish_option);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the DiemRoot role [[H11]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDiemRoot">Roles::AbortsIfNotDiemRoot</a>{account: dr_account};
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_AbortsIfNotPublished">DiemConfig::AbortsIfNotPublished</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;;
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_SetAbortsIf">DiemConfig::SetAbortsIf</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;{account: dr_account};
</code></pre>



</details>

<a name="0x1_DiemTransactionPublishingOption_set_open_module"></a>

## Function `set_open_module`

Allow module publishing from arbitrary sender or not.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_set_open_module">set_open_module</a>(dr_account: &signer, open_module: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_set_open_module">set_open_module</a>(dr_account: &signer, open_module: bool) {
    <a href="Roles.md#0x1_Roles_assert_diem_root">Roles::assert_diem_root</a>(dr_account);

    <b>let</b> publish_option = <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();

    publish_option.module_publishing_allowed = open_module;
    <a href="DiemConfig.md#0x1_DiemConfig_set">DiemConfig::set</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;(dr_account, publish_option);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the DiemRoot role [[H11]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDiemRoot">Roles::AbortsIfNotDiemRoot</a>{account: dr_account};
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_AbortsIfNotPublished">DiemConfig::AbortsIfNotPublished</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;;
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_SetAbortsIf">DiemConfig::SetAbortsIf</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;{account: dr_account};
</code></pre>



</details>

<a name="0x1_DiemTransactionPublishingOption_halt_all_transactions"></a>

## Function `halt_all_transactions`

If called, transactions cannot be sent from any account except DiemRoot


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_halt_all_transactions">halt_all_transactions</a>(dr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_halt_all_transactions">halt_all_transactions</a>(dr_account: &signer) {
    <a href="Roles.md#0x1_Roles_assert_diem_root">Roles::assert_diem_root</a>(dr_account);
    <b>assert</b>(
        !<b>exists</b>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_HaltAllTransactions">HaltAllTransactions</a>&gt;(<a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(dr_account)),
        <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_EHALT_ALL_TRANSACTIONS">EHALT_ALL_TRANSACTIONS</a>),
    );
    move_to(dr_account, <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_HaltAllTransactions">HaltAllTransactions</a> {});
}
</code></pre>



</details>

<a name="0x1_DiemTransactionPublishingOption_resume_transactions"></a>

## Function `resume_transactions`

If called, transactions can be sent from any account once again


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_resume_transactions">resume_transactions</a>(dr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_resume_transactions">resume_transactions</a>(dr_account: &signer) <b>acquires</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_HaltAllTransactions">HaltAllTransactions</a> {
    <a href="Roles.md#0x1_Roles_assert_diem_root">Roles::assert_diem_root</a>(dr_account);
    <b>let</b> dr_address = <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(dr_account);
    <b>assert</b>(
        <b>exists</b>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_HaltAllTransactions">HaltAllTransactions</a>&gt;(dr_address),
        <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_EHALT_ALL_TRANSACTIONS">EHALT_ALL_TRANSACTIONS</a>),
    );

    <b>let</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_HaltAllTransactions">HaltAllTransactions</a> {} = move_from&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_HaltAllTransactions">HaltAllTransactions</a>&gt;(dr_address);
}
</code></pre>



</details>

<a name="0x1_DiemTransactionPublishingOption_transactions_halted"></a>

## Function `transactions_halted`

Return true if all non-administrative transactions are currently halted


<pre><code><b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_transactions_halted">transactions_halted</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_transactions_halted">transactions_halted</a>(): bool {
    <b>exists</b>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_HaltAllTransactions">HaltAllTransactions</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>())
}
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification



<a name="@Initialization_2"></a>

### Initialization



<pre><code><b>invariant</b> [<b>global</b>] <a href="DiemTimestamp.md#0x1_DiemTimestamp_is_operating">DiemTimestamp::is_operating</a>() ==&gt;
    <a href="DiemConfig.md#0x1_DiemConfig_spec_is_published">DiemConfig::spec_is_published</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();
</code></pre>



<a name="@Access_Control_3"></a>

### Access Control

Only <code>set_open_script</code>, and <code>set_open_module</code> can modify the
DiemTransactionPublishingOption config [[H11]][PERMISSION]


<a name="0x1_DiemTransactionPublishingOption_DiemVersionRemainsSame"></a>


<pre><code><b>schema</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_DiemVersionRemainsSame">DiemVersionRemainsSame</a> {
    <b>ensures</b> <b>old</b>(<a href="DiemConfig.md#0x1_DiemConfig_spec_is_published">DiemConfig::spec_is_published</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;()) ==&gt;
        <b>global</b>&lt;<a href="DiemConfig.md#0x1_DiemConfig">DiemConfig</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>()) ==
            <b>old</b>(<b>global</b>&lt;<a href="DiemConfig.md#0x1_DiemConfig">DiemConfig</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>()));
}
</code></pre>




<pre><code><b>apply</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_DiemVersionRemainsSame">DiemVersionRemainsSame</a> <b>to</b> * <b>except</b> set_open_script, set_open_module;
</code></pre>



<a name="@Helper_Functions_4"></a>

### Helper Functions



<a name="0x1_DiemTransactionPublishingOption_spec_is_script_allowed"></a>


<pre><code><b>define</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_spec_is_script_allowed">spec_is_script_allowed</a>(account: signer, hash: vector&lt;u8&gt;): bool {
    <b>let</b> publish_option = <a href="DiemConfig.md#0x1_DiemConfig_spec_get_config">DiemConfig::spec_get_config</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();
    <a href="Roles.md#0x1_Roles_has_diem_root_role">Roles::has_diem_root_role</a>(account) || (!<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_transactions_halted">transactions_halted</a>() && (
        <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(hash) ||
            (<a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(publish_option.script_allow_list)
                || <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_spec_contains">Vector::spec_contains</a>(publish_option.script_allow_list, hash))
    ))
}
<a name="0x1_DiemTransactionPublishingOption_spec_is_module_allowed"></a>
<b>define</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_spec_is_module_allowed">spec_is_module_allowed</a>(account: signer): bool {
    <b>let</b> publish_option = <a href="DiemConfig.md#0x1_DiemConfig_spec_get_config">DiemConfig::spec_get_config</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();
    publish_option.module_publishing_allowed || <a href="Roles.md#0x1_Roles_has_diem_root_role">Roles::has_diem_root_role</a>(account)
}
</code></pre>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/main/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/main/dips/dip-2.md#permissions
