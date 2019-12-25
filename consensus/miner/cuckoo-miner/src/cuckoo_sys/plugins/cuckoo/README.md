Cuck(at)oo Cycle
================

My [Grincon0](https://grincon.org/) talk on [Cuckoo Cycle](https://www.youtube.com/watch?list=PLvgCPbagiHgqYdVUj-ylqhsXOifWrExiq&v=CLiKX0nOsHE)
My [GrinconUS](https://grincon.us/) talk on [Cuckoo Cycle](https://www.youtube.com/watch?v=OsBsz8bKeN4)

[Blog article explaining Cuckoo Cycle](http://cryptorials.io/beyond-hashcash-proof-work-theres-mining-hashing)

[Whitepaper](doc/cuckoo.pdf?raw=true)

Cuckoo Cycle is the first graph-theoretic proof-of-work, and the most memory bound, yet with instant verification.
Unlike Hashcash, Cuckoo Cycle is immune from quantum speedup by Grover's search algorithm.
Cuckatoo Cycle is a variation of Cuckoo Cycle that aims to simplify ASICs by reducing ternary counters to plain bits.

Simplest known PoW
------------------
With a 42-line [complete specification](doc/spec), Cuckatoo Cycle is less than half the size of either
[SHA256](https://en.wikipedia.org/wiki/SHA-2#Pseudocode),
[Blake2b](https://en.wikipedia.org/wiki/BLAKE_%28hash_function%29#Blake2b_algorithm), or
[SHA3 (Keccak)](https://github.com/mjosaarinen/tiny_sha3/blob/master/sha3.c)
as used in Bitcoin, Equihash and ethash. Simplicity matters.

Cuckatoo proofs take the form of a length 42 off-by-1-cycle in a bipartite graph with 2^N+2^N nodes and
2^N edges, with N ranging from 10 up to 64.

In an off-by-1-cycle, consecutive edges are not incident on a single node, but on two nodes that differ in the last bit only.
From now on, whenever we say cycle, we'll mean n off-by-1-cycle.

The graph is defined by the siphash-2-4 keyed hash function mapping an edge index
and partition side (0 or 1) to the edge endpoint on that side.
This makes verification trivial: hash the header to derive the siphash key,
then compute the 42x2 edge endpoints with 84 siphashes,
and that you come back to the starting edge only after traversing 42 off-by-1 incident nodes.

While trivially verifiable, finding a 42-cycle, on the other hand, is far from trivial,
requiring considerable resources, and some luck
(the odds of a random cuckoo graph having an L-cycle are approximately 1 in L).

Lean mining
-----------
The memory efficient miner uses 1 bit per edge and 1 bit per node in one partition
(or 1 bit per 2^k nodes with linear slowdown).
It is bottlenecked by accessing node bits completely at random, making it memory latency bound.
The core of this miner, where over 99% of time is spent, is also [relatively simple](doc/simplesolve).

Mean mining
-----------
The roughly 4x faster latency avoiding miner uses 33 bits per edge
and is bottlenecked by bucket sorting, making it memory bandwidth bound.

Dynamic Sizing
--------------
Instead of fixing N, a proof-of-work system could allow miners to work on any graph size they like,
above a certain minimum. Cycles in larger graphs are more valued as they take more effort to find.
We propose to scale the difficulty for a graph with 2^N edges by 2^(N+1) * N,
which is the number of bits of siphash output that define the graph.

Upgrade by Soft Fork
--------------------
Dynamic Sizing allows for increasing the memory required for mining simply by raising the minimum allowed graph size.
Since this makes new valid blocks a subset of old valid blocks, it is not a hard fork but a soft fork, easily deployed
with majority miner support. Miner manufacturers are incentivized to support larger sizes as being more future proof.

Automatic Upgrades
------------------
A raise in minimum graph size could be triggered automatically if over the last so many blocks (on the scale of months), less than a certain fraction (a minority like 1/3 or 1/4) were solved at that minimum size. This locks in a new minimum which then activates within so many blocks (on the scale of weeks).

ASIC Friendly
-------------
The more efficient lean mining requires tons of SRAM, which is lacking on CPUs and GPUs, but easily implemented in ASICs,
either on a single chip, or for even larger graph sizes (cuckatoo33 requires 1 GB of SRAM), on multiple chips.

Cycle finding
--------------
The most efficient known way to find cycles in random bipartite graphs is
to start by eliminating edges that are not part of any cycle (over 99.9% of edges).
This preprocessing phase is called edge trimming and actually takes the vast majority of runtime.

The algorithm implemented in lean_miner.hpp runs in time linear in 2^N.
(Note that running in sub-linear time is out of the question, as you could
only compute a fraction of all edges, and the odds of all 42 edges of a cycle
occurring in this fraction are astronomically small).
Memory-wise, it uses an 2^N bit edge bitmap of potential cycle edges,
and a 2^N bit node bitmap of nodes with incident edges.

The bandwidth bound algorithm implemented in mean_miner.hpp
uses (1+&Epsilon;) &times; 2^N words to maintain bucket sorted bins of edges instead of a bitmap,
and only uses tiny node bitmaps within a bin to trim edges.

After edge trimming, an standard backtracking graph traversal
is run to recognise all cycles, and report those of the right length.

Performance
--------------
The runtime of a single proof attempt for a 2^29 edge graph on a 4GHz i7-4790K is 10.5 seconds
with the single-threaded mean solver, using 2200MB (or 3200MB with faster solution recovery).
This reduces to 3.5 seconds with 4 threads (3x speedup).

Using an order of magnitude less memory (128MB),
the lean solver takes 32.8 seconds per proof attempt.
Its multi-threading performance is less impressive though,
with 2 threads still taking 25.6 seconds and 4 taking 20.5 seconds.

I claim that siphash-2-4 is a safe choice of underlying hash function,
that these implementations are reasonably optimal,
that trading off (less) memory for (more) running time,
incurs at least one order of magnitude extra slowdown,
and finally, that mean_miner.cu is a reasonably optimal GPU miner.
The latter runs about 10x faster on an NVIDA 1080Ti than mean_miner.cpp on an Intel Core-i7 CPU.
In support of these claims, I offer the following bounties:

CPU Speedup Bounties
--------------------
$10000 for an open source implementation that finds 42-cycles twice as fast
as cuckoo/slean_miner, using no more than 1 byte per edge.

Wilke Trei has submitted a successful (1-1/e)-bounty claim with his slean miner at https://github.com/Lolliedieb/cuckoo

$10000 for an open source implementation that finds 42-cycles twice as fast
as mean_miner, regardless of memory use.

Linear Time-Memory Trade-Off Bounty
-----------------------------------
$10000 for an open source implementation that uses at most 2^N/k bits while finding 42-cycles up to 10 k times slower, for any k>=2.

All of these bounties require N ranging over {27,29,31} and #threads
ranging over {1,2,4,8}, and further assume a high-end Intel Core i7 or Xeon and
recent gcc compiler with regular flags as in my Makefile.

GPU Speedup Bounty
------------------
$5000 for an open source implementation for a consumer GPU
that finds 42-cycles twice as fast as cuckoo/lean_miner.cu on 2^29 edge graphs on comparable hardware, using no more than a nibble per edge.

$5000 for an open source implementation for a consumer GPU
that finds 42-cycles twice as fast as cuckarood/photon.cu on 2^29 edge graphs on comparable hardware.

The Makefile defines corresponding targets leancpubounty, meancpubounty, tmtobounty, and gpubounty.

Double and fractional bounties
------------------------------
Improvements by a factor of 4 will be rewarded with double the regular bounty.

In order to minimize the risk of missing out on less drastic improvements,
I further offer a fraction FRAC of the regular CPU/GPU-speedup bounty, payable in bitcoin cash,
for improvements by a factor of 2^FRAC, where FRAC is at least one-tenth.
Note that 2^0.1 is about a 7% improvement.

Siphash Bounties
----------------
While both siphash-2-4 and siphash-1-3 pass the [smhasher](https://github.com/aappleby/smhasher)
test suite for non-cryptographic hash functions,
siphash-1-2, with 1 compression round and only 2 finalization rounds,
[fails](doc/SipHash12) quite badly in the Avalanche department.
We invite attacks on Cuckoo Cycle's dependence on its underlying hash function by offering

$5000 for an open source implementation that finds 42-cycles in graphs defined by siphash-1-2
twice as fast as lean_miner on graphs defined by siphash-2-4, using no more than 1 byte per edge.

$5000 for an open source implementation that finds 42-cycles in graphs defined by siphash-1-2
twice as fast as mean_miner on graphs defined by siphash-2-4, regardless of memory use.

These bounties are not subject to double and/or fractional payouts.

Happy bounty hunting!
 
How to build
--------------
<pre>
cd src
make
</pre>

Bounty contributors
-------------------

* [Zcash Forum](https://forum.z.cash/) participants
* [Genesis Mining](https://www.genesis-mining.com/)
* [Simply VC](https://www.simply-vc-co.ltd/?page_id=8)
* [Claymore](https://bitcointalk.org/index.php?topic=1670733.0)
* [Aeternity developers](http://www.aeternity.com/)

Projects using Cuckoo Cycle
--------------
* [Grin](https://hq.grin.ninja/)
* [æternity - the oracle machine](http://www.aeternity.com/)
* [BitGrin // MimbleWimble with bitcoin economics](https://bitgrin.io/)
* [CodeChain](https://codechain.io/)
* [BitCash](https://www.choosebitcash.com/)
* [Veres One](https://veres.one)
* [BIP 154: Rate Limiting via peer specified challenges; Bitcoin Peer Services](https://github.com/bitcoin/bips/blob/master/bip-0154.mediawiki)
* [Raddi // radically decentralized discussion](http://www.raddi.net/)
* [Cortex // AI on Blockchain](https://www.cortexlabs.ai/)

Projects reconsidering Cuckoo Cycle
--------------
* [Handshake](https://handshake.org) found [unreconcilable issues](https://handshake.org/files/handshake.txt)

![](img/logo.png?raw=true)
