Progress report on the Cuckoo Cycle GPU solver
============

UPDATE
------
This document is obsoleted (in part) by photon's 4x faster GPU solver from early April 2018.

Since completing my rewrite of xenoncat's performance quadrupling CPU solver (winning a double bounty)
in the form of mean_miner.cpp, I've been slowly grinding away at porting that code to CUDA.

I consider myself an amateur GPU coder, having previously ported the latency bound lean_miner to CUDA
(and having to pay a bounty to fellow Dutchman Genoil for improving performance by merely
tweaking the threads-per-block which I had naively fixed at 1), as well as my own
[Equihash miner](https://github.com/tromp/equihash) submission to the
[Zcash Open Source Miner Challenge](https://z.cash/blog/open-source-miner-winners.html).
That Equihash CUDA miner achieved a paltry 27.2 Sol/s on an NVIDIA GTX 980,
matching the performance of my Equihash CPU solver. But it did serve as the basis for far more capable
rewrites such as this
[Nicehash miner](https://github.com/nicehash/nheqminer/blob/master/cuda_djezo/equi_miner.cu)
by Leet Softdev (a.k.a. djezo), which achieves around 400 Sol/s on similar hardware.

Today, on Jan 30, 2018, I completed work on my CUDA solver, mean_miner.cu
------------

Unlike my other, more generic solvers, this one is written to target only billion-node (2^30 to be precise)
graphs, which is how Cuckoo Cycle will be deployed in the upcoming cryptocurrencies Aeternity and Grin.
The reason I recommended deployment at this size is that it's the largest one that a majority of GPUs
can handle. Indeed, with the -b 64 option, the solver uses 3.95 GB. Throwing more memory at this solver
doesn't help that much; the default setting gains less than 5% while using 5.4GB.
Changing settings to allow it to run within 3GB however imposes a huge penalty, more than halving performance.

All my final tuning was done on an NVIDIA 1080 Ti. The only other cards I ran it on were
a plain 1080, which is about 50% slower, and a GTX 980 Ti, which is over 200% slower...

How fast is this Cuckoo Cycle solver on the fastest known consumer hardware?
------------

First of all, we need to agree on a performance metric. Cuckoo Cycle and Equihash are examples of
asymmetric proofs-of-work. As explained in this
[article](http://cryptorials.io/beyond-hashcash-proof-work-theres-mining-hashing),
instead of just computing hash functions, they look for solutions to puzzles. Now, in the case of
Equihash, solutions are plentiful, on average nearly 2 solutions per random puzzle. So there it makes
sense to measure performance in solutions per second. With Cuckoo Cycle, the 42-cycles solutions
we're looking for are rather rare, so instead it makes more sense to measure performance as puzzles
per second. Since each puzzle is a billion-node graph, we arrive at graphs per second as the appropriate
performance measure. So, to return to our question:

How many graphs per second does the fastest solver achieve?
------------

About one.

cuda_miner.cu takes about 0.98 seconds to search one graph on the NVIDIA 1080 Ti.
That's still 2.5 times faster than mean_miner.cpp on the fastest Intel Core i7 CPU,
and according to nvidia-smi, the GPU was using 155W of power.
I don't know how much the i7 was using, but it was likely more than 155W/2.5 = 62W,
which also makes the GPU the more power efficient solver, although not by much.

Here's a typical solver run:

    $ ./cuda30 -r 2
    GeForce GTX 1080 Ti with 10GB @ 352 bits x 5505MHz
    Looking for 42-cycle on cuckoo30("",0-1) with 50% edges, 128*128 buckets, 256 trims, and 128 thread blocks.
    Using 2680MB bucket memory and 21MB memory per thread block (5392MB total)
    nonce 0 k0 k1 k2 k3 a34c6a2bdaa03a14 d736650ae53eee9e 9a22f05e3bffed5e b8d55478fa3a606d
       4-cycle found
       2-cycle found
      16-cycle found
      26-cycle found
      40-cycle found
    findcycles completed on 38906 edges
    Time: 2215 ms
    nonce 1 k0 k1 k2 k3 be6c0ae25622e409 ede28d78411671d4 74ffaa51c7aa70ac 2ab552193088c87a
       4-cycle found
     282-cycle found
    1006-cycle found
     390-cycle found
    findcycles completed on 33234 edges
    Time: 972 ms
    0 total solutions

We used option -r to specify a range of 2 nonces. For some reason, the first
nonce is always run at a slower pace, so we're more interested in the time
taken by the 2nd nonce. Each nonce is hashed together with a header into a 256
bit key for the siphash function which generates the half-billion graph edges.
This key is shown after each nonce as four 64-bit hexadecimal numbers. The GPU is
responsible for generating all edges and then trimming the majority of
them away for clearly not being part of any cycle. After a default number of
256 trimming rounds, only about 1 in every 13450 edges survives, and the
remaining 37000 or so edges are sent back to the CPU for cycle finding, using a an
algorithm inspired by Cuckoo Hashing (which is where the name derives from).

To see a synopsis of all possible options, run:

    $ ./cuda30 -s
    SYNOPSIS
      cuda30 [-b blocks] [-d device] [-h hexheader] [-k rounds [-c count]] [-m trims] [-n nonce] [-r range] [-U blocks] [-u threads] [-V threads] [-v threads] [-T threads] [-t threads] [-X threads] [-x threads] [-Y threads] [-y threads] [-Z threads] [-z threads]
    DEFAULTS
      cuda30 -b 128 -d 0 -h "" -k 0 -c 1 -m 256 -n 0 -r 1 -U 128 -u 8 -V 32 -v 128 -T 32 -t 128 -X 32 -x 64 -Y 32 -y 128 -Z 32 -z 8


Most of these are for shaping the GPU's thread parallellism in the various edge generation and trimming rounds. 
A detailed discussion of these can be found in this [tuning guide](https://github.com/tromp/cuckoo/blob/master/GPU_tuning.md)

Here's a run that uncovers a solution:

    $ ./cuda30 -n 60 -r 4
    GeForce GTX 1080 Ti with 10GB @ 352 bits x 5505MHz
    Looking for 42-cycle on cuckoo30("",60-63) with 50% edges, 128*128 buckets, 256 trims, and 128 thread blocks.
    Using 2680MB bucket memory and 21MB memory per thread block (5392MB total)
    nonce 60 k0 k1 k2 k3 275f9313c78adcec c3dc47d972920e25 41f8c5d51abbf1e7 74da5cc5b52b2a0b
       6-cycle found
      12-cycle found
       2-cycle found
      16-cycle found
      26-cycle found
       8-cycle found
    findcycles completed on 39931 edges
    Time: 2194 ms
    nonce 61 k0 k1 k2 k3 4178123b6607deb3 596e493a0fe04022 685fbcfc1d315fe 7cf66796fc0083c1
       4-cycle found
      32-cycle found
      56-cycle found
     314-cycle found
     302-cycle found
    findcycles completed on 38585 edges
    Time: 967 ms
    nonce 62 k0 k1 k2 k3 544f44b2b17afc97 4ba38ecebc2fa72c 21e2c32bba7f6196 4d8886ccba77435b
    findcycles completed on 37915 edges
    Time: 969 ms
    nonce 63 k0 k1 k2 k3 7d4f06d5f68dc772 331017080ac63322 e62926ee68af70ed cf2efe7e2f4dbc16
      42-cycle found
      84-cycle found
     192-cycle found
     320-cycle found
    findcycles completed on 36530 edges
    Time: 1084 ms
    Solution 23ece 27e0856 2ad8c27 2cbb0b5 3694cdd 477a095 64de6fc 64e1c92 68e624d 6aa4c6f 6b1d0c2 76f07d2 c273122 c2e38ed c655cde c97ba17 e708130 ec8890d ecb9932 f28d66d f577aff 104d8441 116de91f 116e61cb 1178ea28 11840f8a 11ce10b0 12792630 12ae2388 140ae893 1439b9fd 146a3047 1538d93c 176cb068 17e01c9b 1876ee0a 1c871774 1d37d976 1d6fa785 1d9c1669 1d9d015e 1db85f7e
    Verified with cyclehash b06d3a638f4237c1d5d96fe57e549a83b76421522fb09af24d3520fb91f364f7
    1 total solutions

We can see that recovering the 42-cycle takes some extra time, since the solver doesn't keep track of what
edge index generates what edge endpoint, so it needs to regenerate all of them.


One may wonder:

Does it really take a second to solve this puzzle?
------------

No, I don't believe that for a second! I think my solver is suboptimal for several reasons, including:

1. Uses memcpy() to perform unaligned reads/writes of 40-bit and 48-bit numbers
2. Uses only a fraction of the available memory bandwidth
3. Ignorant of warp boundaries
4. Makes no use of PTX assembly
5. Written by a GPU coding amateur

So I suspect there's room for at least a doubling of performance, and perhaps even a quadrupling
(I don't expect to see more than single digit performances though).
To that end I offer the GPU speedup bounty on the [Cuckoo Cycle homepage](https://github.com/tromp/cuckoo)

Please help me find the remaning possible optimizations and perhaps help yourself to some Bitcoin Cash
in the process!
