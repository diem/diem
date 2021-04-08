---
id: overview
title: SDKs
hide_right_sidebar: true
---

Our official SDKs are collections of development resources like libraries, code samples, and documentation curated to help you build your own projects on the Diem platform.

Select a language to access its approved Diem resource package on GitHub:

## Select a language:

<CardsWrapper cardsPerRow={2}>
  <SDKCard
    docs="https://javadoc.io/doc/com.diem/client-sdk-java/latest/index.html"
    icon="/img/docs/sdk-java.png"
    sdk="https://github.com/libra/client-sdk-java"
  />
  <SDKCard
    docs="https://godoc.org/github.com/diem/client-sdk-go"
    icon="/img/docs/sdk-go.png"
    sdk="https://github.com/libra/client-sdk-go"
  />
  <SDKCard
    docs="https://diem.github.io/client-sdk-python/diem/index.html"
    icon="/img/docs/sdk-python.png"
    sdk="https://github.com/libra/client-sdk-python"
  />
</CardsWrapper>

## Coming soon:

<CardsWrapper>
  <SimpleTextCard
    icon="/img/docs/rust-alt.png"
    iconDark="/img/docs/rust-alt-dark.png"
    overlay="Diem Rust Crate list and documentation"
    title="Rust Docs"
    to="https://diem.github.io/diem"
  />
</CardsWrapper>

## Warning

Notice that the SDKs are assuming that you have a trusted fullnode providing you with the required JSON-RPC API. The JSON-RPC responses are not providing any proofs that would allow you to verify they contain data that can be trusted.

If you are connecting to a public fullnode that you do not own, it is not providing you the same security guarantees as if you were running your own, allowing it to easily feed you with fake data.
We strongly recommend running your own fullnode as soon as you depend on the JSON-RPC API or the SDKs for any critical operation.
