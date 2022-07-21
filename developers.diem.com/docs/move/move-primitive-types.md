---
title: "Primitive types"
id: move-primitive-types
hidden: false
---
<CardsWrapper cardsPerRow={2}>
    <OverlayCard
        to="/docs/move/move-primitive-types/move-primitives-integers"
        icon="img/integers-bool.svg"
        iconDark="img/integers-bool-dark.svg" 
        title="Integers"
        description="Move supports three unsigned integer types: u8, u64, and u128"
    />
    <OverlayCard
        to="/docs/move/move-primitive-types/move-primitives-bool"
        icon="img/integers-bool.svg"
        iconDark="img/integers-bool-dark.svg" 
        title="Bool"
        description="Bool is Move's primitive type for boolean true and false values."
    />
    <OverlayCard
        to="/docs/move/move-primitive-types/move-primitives-address"
        icon="img/address.svg"
        iconDark="img/address-dark.svg" 
        title="Address"
        description="Address is a built-in type in Move that is used to represent locations
        in global storage"
    />
    <OverlayCard
        to="/docs/move/move-primitive-types/move-primitives-vector"
        icon="img/vector.svg"
        iconDark="img/vector-dark.svg" 
        title="Vector"
        description="Vector&lt;T&gt; is the only primitive collection type provided by Move"
    />
    <OverlayCard
        to="/docs/move/move-primitive-types/move-primitives-signer"
        icon="img/signer.svg"
        iconDark="img/signer-dark.svg" 
        title="Signer"
        description="Signer is a built-in Move resource type. A signer is a capability that
        allows the holder to act on behalf of a particular address"
    />
    <OverlayCard
        to="/docs/move/move-primitive-types/move-primitives-references"
        icon="img/move-references.svg"
        iconDark="img/move-references-dark.svg" 
        title="References"
        description="Move has two types of references: immutable &amp; and mutable."
    />
    <OverlayCard
        to="/docs/move/move-primitive-types/move-primitives-tuples-unit"
        icon="img/tuples.svg"
        iconDark="img/tuples-dark.svg" 
        title="Tuples and Unit"
        description="In order to support multiple return values, Move has tuple-like
        expressions. We can consider unit() to be an empty tuple"
    />
</CardsWrapper>