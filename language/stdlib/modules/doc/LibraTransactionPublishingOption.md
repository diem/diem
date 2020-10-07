
<a name="0x1_LibraTransactionPublishingOption"></a>

# Module `0x1::LibraTransactionPublishingOption`

This module defines a struct storing the publishing policies for the VM.


-  [Struct `LibraTransactionPublishingOption`](#0x1_LibraTransactionPublishingOption_LibraTransactionPublishingOption)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_LibraTransactionPublishingOption_initialize)
-  [Function `is_script_allowed`](#0x1_LibraTransactionPublishingOption_is_script_allowed)
-  [Function `is_module_allowed`](#0x1_LibraTransactionPublishingOption_is_module_allowed)
-  [Function `add_to_script_allow_list`](#0x1_LibraTransactionPublishingOption_add_to_script_allow_list)
-  [Function `set_open_script`](#0x1_LibraTransactionPublishingOption_set_open_script)
-  [Function `set_open_module`](#0x1_LibraTransactionPublishingOption_set_open_module)
-  [Module Specification](#@Module_Specification_1)
    -  [Initialization](#@Initialization_2)
    -  [Access Control](#@Access_Control_3)
    -  [Helper Functions](#@Helper_Functions_4)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="LibraConfig.md#0x1_LibraConfig">0x1::LibraConfig</a>;
<b>use</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp">0x1::LibraTimestamp</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_LibraTransactionPublishingOption_LibraTransactionPublishingOption"></a>

## Struct `LibraTransactionPublishingOption`

Defines and holds the publishing policies for the VM. There are three possible configurations:
1. No module publishing, only allow-listed scripts are allowed.
2. No module publishing, custom scripts are allowed.
3. Both module publishing and custom scripts are allowed.
We represent these as the following resource.


<pre><code><b>struct</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_LibraTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT"></a>

The script hash already exists in the allowlist


<pre><code><b>const</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT">EALLOWLIST_ALREADY_CONTAINS_SCRIPT</a>: u64 = 1;
</code></pre>



<a name="0x1_LibraTransactionPublishingOption_EINVALID_SCRIPT_HASH"></a>

The script hash has an invalid length


<pre><code><b>const</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_EINVALID_SCRIPT_HASH">EINVALID_SCRIPT_HASH</a>: u64 = 0;
</code></pre>



<a name="0x1_LibraTransactionPublishingOption_SCRIPT_HASH_LENGTH"></a>



<pre><code><b>const</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_SCRIPT_HASH_LENGTH">SCRIPT_HASH_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x1_LibraTransactionPublishingOption_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_initialize">initialize</a>(lr_account: &signer, script_allow_list: vector&lt;vector&lt;u8&gt;&gt;, module_publishing_allowed: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_initialize">initialize</a>(
    lr_account: &signer,
    script_allow_list: vector&lt;vector&lt;u8&gt;&gt;,
    module_publishing_allowed: bool,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);

    <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config">LibraConfig::publish_new_config</a>(
        lr_account,
        <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a> {
            script_allow_list, module_publishing_allowed
        }
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the LibraRoot role [[H10]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_PublishNewConfigAbortsIf">LibraConfig::PublishNewConfigAbortsIf</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_PublishNewConfigEnsures">LibraConfig::PublishNewConfigEnsures</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt; {
    payload: <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a> {
        script_allow_list, module_publishing_allowed
    }};
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_is_script_allowed"></a>

## Function `is_script_allowed`

Check if sender can execute script with <code>hash</code>


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_is_script_allowed">is_script_allowed</a>(account: &signer, hash: &vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_is_script_allowed">is_script_allowed</a>(account: &signer, hash: &vector&lt;u8&gt;): bool {
    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();

    <a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&publish_option.script_allow_list)
        || <a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(&publish_option.script_allow_list, hash)
        || <a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(account)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_AbortsIfNoTransactionPublishingOption">AbortsIfNoTransactionPublishingOption</a>;
</code></pre>




<a name="0x1_LibraTransactionPublishingOption_AbortsIfNoTransactionPublishingOption"></a>


<pre><code><b>schema</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_AbortsIfNoTransactionPublishingOption">AbortsIfNoTransactionPublishingOption</a> {
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>() ==&gt; <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">LibraConfig::AbortsIfNotPublished</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;{};
}
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_is_module_allowed"></a>

## Function `is_module_allowed`

Check if a sender can publish a module


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_is_module_allowed">is_module_allowed</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_is_module_allowed">is_module_allowed</a>(account: &signer): bool {
    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();

    publish_option.module_publishing_allowed || <a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(account)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_AbortsIfNoTransactionPublishingOption">AbortsIfNoTransactionPublishingOption</a>;
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_add_to_script_allow_list"></a>

## Function `add_to_script_allow_list`

Add <code>new_hash</code> to the list of script hashes that is allowed to be executed by the network.


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_add_to_script_allow_list">add_to_script_allow_list</a>(lr_account: &signer, new_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_add_to_script_allow_list">add_to_script_allow_list</a>(lr_account: &signer, new_hash: vector&lt;u8&gt;) {
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);

    <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&new_hash) == <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_SCRIPT_HASH_LENGTH">SCRIPT_HASH_LENGTH</a>, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_EINVALID_SCRIPT_HASH">EINVALID_SCRIPT_HASH</a>));

    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();
    <b>if</b> (<a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(&publish_option.script_allow_list, &new_hash)) {
          <b>abort</b> <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT">EALLOWLIST_ALREADY_CONTAINS_SCRIPT</a>)
    };
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> publish_option.script_allow_list, new_hash);

    <a href="LibraConfig.md#0x1_LibraConfig_set">LibraConfig::set</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;(lr_account, publish_option);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the LibraRoot role [[H10]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<a name="0x1_LibraTransactionPublishingOption_allow_list$8"></a>
<b>let</b> allow_list = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;().script_allow_list;
<b>aborts_if</b> <a href="Vector.md#0x1_Vector_length">Vector::length</a>(new_hash) != <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_SCRIPT_HASH_LENGTH">SCRIPT_HASH_LENGTH</a> <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>aborts_if</b> <a href="Vector.md#0x1_Vector_spec_contains">Vector::spec_contains</a>(allow_list, new_hash) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">LibraConfig::AbortsIfNotPublished</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_SetAbortsIf">LibraConfig::SetAbortsIf</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;{account: lr_account};
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_set_open_script"></a>

## Function `set_open_script`

Allow the execution of arbitrary script or not.


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_set_open_script">set_open_script</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_set_open_script">set_open_script</a>(lr_account: &signer) {
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();

    publish_option.script_allow_list = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();
    <a href="LibraConfig.md#0x1_LibraConfig_set">LibraConfig::set</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;(lr_account, publish_option);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the LibraRoot role [[H10]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">LibraConfig::AbortsIfNotPublished</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_SetAbortsIf">LibraConfig::SetAbortsIf</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;{account: lr_account};
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_set_open_module"></a>

## Function `set_open_module`

Allow module publishing from arbitrary sender or not.


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_set_open_module">set_open_module</a>(lr_account: &signer, open_module: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_set_open_module">set_open_module</a>(lr_account: &signer, open_module: bool) {
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);

    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();

    publish_option.module_publishing_allowed = open_module;
    <a href="LibraConfig.md#0x1_LibraConfig_set">LibraConfig::set</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;(lr_account, publish_option);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the LibraRoot role [[H10]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">LibraConfig::AbortsIfNotPublished</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_SetAbortsIf">LibraConfig::SetAbortsIf</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;{account: lr_account};
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification



<a name="@Initialization_2"></a>

### Initialization



<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt;
    <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();
</code></pre>



<a name="@Access_Control_3"></a>

### Access Control

Only <code>add_to_script_allow_list</code>, <code>set_open_script</code>, and <code>set_open_module</code> can modify the
LibraTransactionPublishingOption config [[H10]][PERMISSION]


<a name="0x1_LibraTransactionPublishingOption_LibraVersionRemainsSame"></a>


<pre><code><b>schema</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_LibraVersionRemainsSame">LibraVersionRemainsSame</a> {
    <b>ensures</b> <b>old</b>(<a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;()) ==&gt;
        <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) ==
            <b>old</b>(<b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()));
}
</code></pre>




<pre><code><b>apply</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_LibraVersionRemainsSame">LibraVersionRemainsSame</a> <b>to</b> * <b>except</b> add_to_script_allow_list, set_open_script, set_open_module;
</code></pre>



<a name="@Helper_Functions_4"></a>

### Helper Functions



<a name="0x1_LibraTransactionPublishingOption_spec_is_script_allowed"></a>


<pre><code><b>define</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_spec_is_script_allowed">spec_is_script_allowed</a>(account: signer, hash: vector&lt;u8&gt;): bool {
    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_spec_get_config">LibraConfig::spec_get_config</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();
    <a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(publish_option.script_allow_list)
        || <a href="Vector.md#0x1_Vector_spec_contains">Vector::spec_contains</a>(publish_option.script_allow_list, hash)
        || <a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(account)
}
<a name="0x1_LibraTransactionPublishingOption_spec_is_module_allowed"></a>
<b>define</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_spec_is_module_allowed">spec_is_module_allowed</a>(account: signer): bool {
    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_spec_get_config">LibraConfig::spec_get_config</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();
    publish_option.module_publishing_allowed || <a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(account)
}
</code></pre>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md
[ROLE]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#roles
[PERMISSION]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#permissions
