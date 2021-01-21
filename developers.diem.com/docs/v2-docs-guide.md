---
id: v2-docs-guide
title: V2 Docs Guide
sidebar_label: V2 Docs Guide
---

<br />

## Excerpt

<Excerpt image="img/white-paper-excerpt.svg">
  The world truly needs a reliable digital currency and infrastructure that together can deliver on the promise of “the internet of money.” Securing your financial assets on your mobile device should be simple and intuitive. Moving money around globally should be as easy and cost-effective as — and even more safe and secure than — sending a text message or sharing a photo, no matter where you live, what you do, or how much you earn.
   &nbsp;<a href='#'>— Diem White Paper</a>
</Excerpt>

## Video Embeds

<VideoEmbed src="https://www.youtube.com/embed/dQw4w9WgXcQ" />

```<VideoEmbed src="https://www.youtube.com/embed/dQw4w9WgXcQ" />```

The above is an example of how to embed a video. If you get the link from youtube, **make sure to get the embed link, which is different than the regular url.

To get it click share, then embed, and you'll see something like this:

```<iframe width="560" height="315" src="https://www.youtube.com/embed/dQw4w9WgXcQ" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>```

You can get the link from there

## Cards

### Color Card

<CardsWrapper>
  <ColorCard
    color="purpleDark"
    icon="img/transaction.svg"
    iconDark="img/transaction-dark.svg"
    to="#"
    title="Send a test transaction"
  />
</CardsWrapper>

### Overlay Card

<CardsWrapper>
  <OverlayCard
    description="I want to understand nodes"
    icon="img/wallet-app.svg"
    iconDark="img/wallet-app-dark.svg"
    title="Nodes"
    to="#"
  />
</CardsWrapper>

### Simple Card

<CardsWrapper>
  <SimpleTextCard
    icon="img/github.svg"
    iconDark="img/github-dark.svg"
    title="Read the core specifications"
    to="#"
  />
</CardsWrapper>

### Tag Card

<CardsWrapper>
  <TagCard
    icon="img/github.svg"
    iconDark="img/github-dark.svg"
    tags={["Web", "Mobile", "Merchant"]}
    title="Reference Wallet"
    to="https://github.com/diem"
  />
</CardsWrapper>

## Reference Section Component

<CardsWrapper>
  <CoreReference />
  <MerchantReference />
  <WalletReference />
  <MoveReference />
  <NodeReference />
</CardsWrapper>

<WaveBackground />

## Normal Code Snippet (and the start of WaveBackground)


```jsx
import React, { useState } from "react";

function Example() {
  const [count, setCount] = useState(0);

  return (
    <div>
      <p>You clicked {count} times</p>
      <button onClick={() => setCount(count + 1)}>
        Click me
      </button>
    </div>
  );
}
```

## Excerpt (with WaveBackground)

<Excerpt image="img/white-paper-excerpt.svg">
  The world truly needs a reliable digital currency and infrastructure that together can deliver on the promise of “the internet of money.” Securing your financial assets on your mobile device should be simple and intuitive. Moving money around globally should be as easy and cost-effective as — and even more safe and secure than — sending a text message or sharing a photo, no matter where you live, what you do, or how much you earn.
   &nbsp;<a href='#'>— Diem White Paper</a>
</Excerpt>
