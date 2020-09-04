---
author: Michael Engle, Libra Association
title: Libra Developer Spotlight
---
<script>
    let items = document.getElementsByClassName("post-meta");   
    for (var i = items.length - 1; i >= 0; i--) {
        if (items[i].innerHTML = '<p class="post-meta">August 14, 2019</p>') items[i].innerHTML = '<p class="post-meta">September 12, 2019</p>';
    }
    var slug = location.pathname.slice(location.pathname.lastIndexOf('/')+1);
    var redirect = 'https://libra.org/en-US/blog/' + slug;
    window.location = redirect;    
</script>

We’re so excited to see so many leading developers from around the globe innovating and working on the Libra testnet. We wanted to share a few examples that caught our eye!

### Transactions

[Mike Kelly](https://twitter.com/mikekelly85/status/1141491588180447233), Founder of fintech app [PayWithCurl](https://paywithcurl.com), [in this Twitter thread](https://twitter.com/mikekelly85/status/1141491022163324929), showed off a demo Libra wallet and range of test-mode payments applications he and his team had built on the platform: 

<div style="margin:3em;">
<blockquote class="twitter-tweet tw-align-center" data-lang="en"><p lang="en" dir="ltr">The <a href="https://twitter.com/paywithcurl?ref_src=twsrc%5Etfw">@paywithcurl</a> team decided to do some building on top of <a href="https://twitter.com/Libra_?ref_src=twsrc%5Etfw">@Libra_</a> and take the testnet for a spin. This is what we’ve built so far (thread)<br><br>Firstly, a Libra wallet with an OAuth API for payments. Here’s me signing up to Curl and connecting my Libra wallet <a href="https://t.co/x8R6tC3fFR">pic.twitter.com/x8R6tC3fFR</a></p>&mdash; Mike Kelly (@mikekelly85) <a href="https://twitter.com/mikekelly85/status/1141491022163324929?ref_src=twsrc%5Etfw">June 19, 2019</a></blockquote>
<script async src="https://platform.twitter.com/widgets.js" charset="utf-8"></script>
</div>

> _“From our builder’s standpoint: Libra is obviously still very early days and there are some rough edges, but the tooling and docs for getting started and poking around are very good. A lot of attention to detail. If this is the future of bank accounts and payments, happy days!”_

Though a block explorer wasn’t released at launch, developers were quick to get to work, with a polished solution for easily scanning transactions cropping up seemingly overnight. [Gal Diskin](https://twitter.com/gal_diskin) is to thank for the creation and maintenance of [Librabrowser.io](https://librabrowser.io/).

> _"It was exciting to build a tool to support the developer community which was why I got hooked up on it and worked all night the first day to deliver it. I want to thank all the contributors that reached out and helped improve [librabrowser.io](https://librabrowser.io/) after it's creation, it is really great to see the power of community around Libra development."_

Other community members have released Libra block explorers for the testnet – these include [LibraVista](https://www.libravista.com/), [Libranaut](https://libranaut.io/), and [Libratics](http://libratics.com/#/home), amongst others.

### Dev Tools and Beyond

Those transacting on the testnet are building some of the earliest Libra applications – software engineer Agro Rachmatullah, was quick to release one of the first iOS Libra wallets, as he demonstrates [in this video](https://twitter.com/agro1986/status/1141194274089783296).

<div style="margin:3em;">
<blockquote class="twitter-tweet tw-align-center" data-lang="en"><p lang="en" dir="ltr">World&#39;s first unofficial iOS <a href="https://twitter.com/hashtag/libra?src=hash&amp;ref_src=twsrc%5Etfw">#libra</a> wallet, made in 1 night lol. Transfer time is so fast... 😲 Libra is not completely open like Bitcoin and Ethereum, but it&#39;s a step in the right direction to show the potential of cryptocurrency &amp; decentralized finance <a href="https://t.co/WqrK77Hncx">pic.twitter.com/WqrK77Hncx</a></p>&mdash; Agro Rachmatullah (ラクマ) (@agro1986) <a href="https://twitter.com/agro1986/status/1141194274089783296?ref_src=twsrc%5Etfw">June 19, 2019</a></blockquote>
<script async src="https://platform.twitter.com/widgets.js" charset="utf-8"></script>
</div>


> _"World's first unofficial iOS #libra wallet, made in 1 night...Transfer time is so fast… 😲"_

For users that simply wish to try out Libra without setting up a client, a disposable web wallet created by [Kulap](https://www.kulap.io/) can be tried out [here](https://dev.kulap.io/libra/#/) – it even comes with [an extensive guide](https://medium.com/kulapofficial/the-first-libra-wallet-poc-building-your-own-wallet-and-apis-3cb578c0bd52) on building your own. Users can create their wallets (with 100 inactive Test Libra Coins) and just open a web page to send it to their friends via QR Code! 

Nattapon Nimakul, Smart Contract Developer at Kulap.io said; 

> _"This POC Libra wallet focusses on users to show how it works by allowing them to play it on their smartphone, laptop or desktop."_

It is equally important to recognise the work done by talented developers to lower the barriers to entry for others – for those more comfortable programming in traditional languages, gRPC clients have surfaced ([JavaScript](https://community.libra.org/t/libra-grpc-the-libra-client-for-javascript-lovers/691), [Python](https://github.com/egorsmkv/libra-grpc-py), [Go](https://github.com/codemaveric/libra-go)), and Docker aficionados will be happy to hear that full-stack dev [Michael Pambo Ognana](https://twitter.com/_mikamboo) has [released an image](https://gist.github.com/mikamboo/2bc9ca3b756fd77c707e59033a475d2e) for running through your first Libra testnet transaction. 

Other notable mentions include the [first token launched on Libra](https://community.libra.org/t/implemented-basic-token-using-move-ir/526/6), [REST APIs](https://github.com/bonustrack/libra-api) and integration into [ChainIDE](https://medium.com/iost/the-worlds-first-online-ide-supporting-libra-now-launched-connecting-facebook-and-iost-3b2ceb013812). 

The gRPC endpoints exposed by the Libra Core client have allowed for great flexibility when it comes to programming. Members of the community have been quick to release implementations of clients in a range of languages, so that developers could rapidly start building Libra apps in [Go](http://github.com/philippgille/libra-sdk-go), [Python](http://github.com/bandprotocol/pylibra), [Java](http://github.com/ketola/jlibra), [PHP](http://github.com/connected-io/php-libra), [JavaScript](http://github.com/bandprotocol/libra-web) and [C#](http://github.com/Rio-dapp/libra-csharp-sdk).

Need somewhere to spend your testnet coins? Try out Kulap’s [Libra Coffee](https://www.libracoffee.io/) e-commerce prototype.  

### How to Get Involved

We’ve been overwhelmed by the positivity and interest in the Libra, and we look forward to working collaboratively with our developer community to see more innovative applications evolving in the coming weeks, months and years. 

Newcomers seeking to get involved with the community are encouraged to join the official Libra Developer Community [here](https://community.libra.org/), where users and Libra engineers frequently answer technical questions. Don’t forget to check out the [comprehensive documentation](https://developers.libra.org/docs/welcome-to-libra) on our website made available for developers looking to get started. Last but not least, stay up to date with all of the latest news and developments on [Twitter](https://twitter.com/libradev) and [GitHub](https://github.com/libra/libra).
