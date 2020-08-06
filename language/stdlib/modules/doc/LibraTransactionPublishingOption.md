
<a name="0x1_LibraTransactionPublishingOption"></a>

# Module `0x1::LibraTransactionPublishingOption`

### Table of Contents

-  [Struct `LibraTransactionPublishingOption`](#0x1_LibraTransactionPublishingOption_LibraTransactionPublishingOption)
-  [Function `initialize`](#0x1_LibraTransactionPublishingOption_initialize)
-  [Function `is_script_allowed`](#0x1_LibraTransactionPublishingOption_is_script_allowed)
-  [Function `is_module_allowed`](#0x1_LibraTransactionPublishingOption_is_module_allowed)
-  [Function `add_to_script_allow_list`](#0x1_LibraTransactionPublishingOption_add_to_script_allow_list)
-  [Function `set_open_script`](#0x1_LibraTransactionPublishingOption_set_open_script)
-  [Function `set_open_module`](#0x1_LibraTransactionPublishingOption_set_open_module)



<a name="0x1_LibraTransactionPublishingOption_LibraTransactionPublishingOption"></a>

## Struct `LibraTransactionPublishingOption`

Defines and holds the publishing policies for the VM. There are three possible configurations:
1. No module publishing, only allowlisted scripts are allowed.
2. No module publishing, custom scripts are allowed.
3. Both module publishing and custom scripts are allowed.
We represent these as the following resource.


<pre><code><b>struct</b> <a href="#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>script_allow_list: vector&lt;vector&lt;u8&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>module_publishing_allowed: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraTransactionPublishingOption_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionPublishingOption_initialize">initialize</a>(lr_account: &signer, script_allow_list: vector&lt;vector&lt;u8&gt;&gt;, module_publishing_allowed: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionPublishingOption_initialize">initialize</a>(
    lr_account: &signer,
    script_allow_list: vector&lt;vector&lt;u8&gt;&gt;,
    module_publishing_allowed: bool,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);

    <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config">LibraConfig::publish_new_config</a>(
        lr_account,
        <a href="#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a> {
            script_allow_list, module_publishing_allowed
        }
    );
}
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_is_script_allowed"></a>

## Function `is_script_allowed`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionPublishingOption_is_script_allowed">is_script_allowed</a>(account: &signer, hash: &vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionPublishingOption_is_script_allowed">is_script_allowed</a>(account: &signer, hash: &vector&lt;u8&gt;): bool {
    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();

    <a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&publish_option.script_allow_list)
        || <a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(&publish_option.script_allow_list, hash)
        || <a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(account)
}
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_is_module_allowed"></a>

## Function `is_module_allowed`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionPublishingOption_is_module_allowed">is_module_allowed</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionPublishingOption_is_module_allowed">is_module_allowed</a>(account: &signer): bool {
    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();

    publish_option.module_publishing_allowed || <a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(account)
}
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_add_to_script_allow_list"></a>

## Function `add_to_script_allow_list`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionPublishingOption_add_to_script_allow_list">add_to_script_allow_list</a>(lr_account: &signer, new_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionPublishingOption_add_to_script_allow_list">add_to_script_allow_list</a>(lr_account: &signer, new_hash: vector&lt;u8&gt;) {
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);

    <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&new_hash) == SCRIPT_HASH_LENGTH, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(EINVALID_SCRIPT_HASH));

    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();
    <b>if</b> (<a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(&publish_option.script_allow_list, &new_hash)) {
          <b>abort</b> <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(EALLOWLIST_ALREADY_CONTAINS_SCRIPT)
    };
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> publish_option.script_allow_list, new_hash);

    <a href="LibraConfig.md#0x1_LibraConfig_set">LibraConfig::set</a>&lt;<a href="#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;(lr_account, publish_option);
}
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_set_open_script"></a>

## Function `set_open_script`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionPublishingOption_set_open_script">set_open_script</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionPublishingOption_set_open_script">set_open_script</a>(lr_account: &signer) {
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();

    publish_option.script_allow_list = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();
    <a href="LibraConfig.md#0x1_LibraConfig_set">LibraConfig::set</a>&lt;<a href="#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;(lr_account, publish_option);
}
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_set_open_module"></a>

## Function `set_open_module`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionPublishingOption_set_open_module">set_open_module</a>(lr_account: &signer, open_module: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionPublishingOption_set_open_module">set_open_module</a>(lr_account: &signer, open_module: bool) {
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);

    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();

    publish_option.module_publishing_allowed = open_module;
    <a href="LibraConfig.md#0x1_LibraConfig_set">LibraConfig::set</a>&lt;<a href="#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;(lr_account, publish_option);
}
</code></pre>



</details>
