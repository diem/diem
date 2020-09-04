---
author: Mathieu Baudet, Calibra
title: Improving the LibraBFT protocol
---
<script>
    let items = document.getElementsByClassName("post-meta");   
    for (var i = items.length - 1; i >= 0; i--) {
        if (items[i].innerHTML = '<p class="post-meta">August 14, 2019</p>') items[i].innerHTML = '<p class="post-meta">September 26, 2019</p>';
    }
    var slug = location.pathname.slice(location.pathname.lastIndexOf('/')+1);
    var redirect = 'https://libra.org/en-US/blog/' + slug;
    window.location = redirect;    
</script>

We are happy to announce a new release of the [LibraBFT technical report](https://developers.libra.org/docs/state-machine-replication-paper).

The LibraBFT protocol operates at the heart of the Libra blockchain to guarantee secure state machine replication. The new version of the protocol, LibraBFTv2, includes several optimizations that were designed to reduce networking and improve commit latency of the Libra blockchain.

Together with this release, we are happy to make available the code of the Rust simulator used as a reference in the LibraBFT report. This code can be found in the [github repository](https://github.com/calibra/research) of the research team of Calibra.

### Reducing network complexity in practice

LibraBFT is a refinement of the HotStuff protocol that makes explicit the mechanisms used to achieve round synchronization between nodes. Informally, a round is a period of time where a specific leader is trusted to drive progress --- typically by proposing a block (B), gathering votes (V), and broadcasting a quorum certificate (C) (see picture below). Round synchronization aims at making nodes eventually execute the same round with sufficiently long overlap so that the leader of this round can succeed.

![](https://libra.org/wp-content/uploads/2019/09/libraBFT2.png)

In the optimistic case (aka "happy path"), *LibraBFTv2 reduces the
overhead of round synchronization to a single message per node per
round* (see green arrows in the picture).

The initial "v1" version of LibraBFT relied on probabilistic gossip to ensure uniform propagation of quorum certificates (C). This uniform propagation was needed to achieve round synchronization and guarantee liveness in presence of malicious leaders. While probabilistic gossip is a popular technique suitable to many applications, it typically requires a non-linear number of messages and causes increased latency due to the intermediate hops. From an engineering point of view, the network overhead and the probabilistic nature of gossiping may also complicate debugging.

In contrast, LibraBFTv2 achieves round synchronization in a different way, without using probabilistic gossip. First, the new protocol introduces a new type of failsafe mechanisms that regularly pull missing data in case no progress is made. Second, LibraBFTv2 simplifies the constraints on block proposals. The new constraints ensure that an honest leader can always propose a block and force round synchronization soon after the first honest node enters her round.

The proof of liveness of LibraBFTv2 shows that the new protocol still performs in a satisfying way under Byzantine (worst-case) scenarios, while the number of messages is now linear in the best case.

### What's next

We expect new releases of the LibraBFT report to continue in the future as the research and the engineering teams of Calibra keep improving the theoretical analysis and the implementation of the LibraBFT protocol.

Stay tuned!
