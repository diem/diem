
<a name="SCRIPT"></a>

# Script `add_currency.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(exchange_rate_denom: u64, exchange_rate_num: u64, is_synthetic: bool, scaling_factor: u64, fractional_part: u64, currency_code: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;NewCurrency&gt;(
    exchange_rate_denom: u64,
    exchange_rate_num: u64,
    is_synthetic: bool,
    scaling_factor: u64,
    fractional_part: u64,
    currency_code: vector&lt;u8&gt;,
) {
    <b>let</b> rate = <a href="../../modules/doc/fixedpoint32.md#0x0_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(
        exchange_rate_denom,
        exchange_rate_num,
    );
    <a href="../../modules/doc/libra.md#0x0_Libra_register_currency">Libra::register_currency</a>&lt;NewCurrency&gt;(
        rate,
        is_synthetic,
        scaling_factor,
        fractional_part,
        currency_code,
    );
}
</code></pre>



</details>
