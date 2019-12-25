Tuning the cuckoo GPU solver
============

Outputs will vary for cuckaroo and cuckatoo but to the extent that options are accepted, results will mostly still be applicable.

Running with option -s provides a synopsis of solver options:

SYNOPSIS
  cuda29 [-s] [-c] [-d device] [-E 0-2] [-h hexheader] [-m trims] [-n nonce] [-r range] [-U seedAblocks] [-u seedAthreads] [-v seedBthreads] [-w Trimthreads] [-y Tailthreads] [-Z recoverblocks] [-z recoverthreads]
DEFAULTS
  cuda29 -d 0 -E 0 -h "" -m 176 -n 0 -r 1 -U 4096 -u 256 -v 128 -w 512 -y 1024 -Z 1024 -z 1024

Let's look at each of these in turn.

-d device
------------
This option lets us select among multiple CUDA devices. Default is 0.
Besides a 1080 Ti as device 0, I also have a plain 1080 available:

    $ ./cuda29 -d 1 | head -1
    GeForce GTX 1080 with 8114MB @ 256 bits x 5005MHz

-h hexheader
------------
This allows specification of arbitrary headers. Default "".
The given string is padded with nul bytes to a fixed header length of 80 bytes.
For example,

    $ ./cuda29 -h "DEADBEEF" | head -2
    GeForce GTX 1080 Ti with 10GB @ 352 bits x 5505MHz
    Looking for 42-cycle on cuckoo30("ޭ??",0) with 50% edges, 64*64 buckets, 176 trims, and 64 thread blocks.

    $ ./cuda29 -h "444541440A42454546" | head -3
    GeForce GTX 1080 Ti with 10GB @ 352 bits x 5505MHz
    Looking for 42-cycle on cuckoo30("DEAD
    BEEF",0) with 50% edges, 64*64 buckets, 176 trims, and 64 thread blocks.

    0 total solutions

-E edge expand round
------------
round before which edges are expanded into explicit endpoints, taking 8 bytes each
Higher values (1 and 2) result in larger memory savings, at the cost of having to do more endpoint recomputation.
For cuckoo30, we can mine in either 7 GB, 5 GB, or as little as 4 GB:

    $ ./cuda29 -E 0 | head -3
    GeForce GTX 1080 Ti with 10GB @ 352 bits x 5505MHz
    Looking for 42-cycle on cuckoo30("",0) with 50% edges, 64*64 buckets, 176 trims, and 64 thread blocks.
    Using 6976MB of global memory.
    $ ./cuda29 -E 1 | head -3
    GeForce GTX 1080 Ti with 10GB @ 352 bits x 5505MHz
    Looking for 42-cycle on cuckoo30("",0) with 50% edges, 64*64 buckets, 176 trims, and 64 thread blocks.
    Using 4848MB of global memory.
    Using 4848MB of global memory.
    $ ./cuda29 -E 2 | head -3
    GeForce GTX 1080 Ti with 10GB @ 352 bits x 5505MHz
    Looking for 42-cycle on cuckoo30("",0) with 50% edges, 64*64 buckets, 176 trims, and 64 thread blocks.
    Using 3488MB of global memory.

-m trims
------------
The number of trimming rounds. Default 176. Can be increased arbitrarily. At some point, there will be
no edges left to trim, as all remaining edges are already part of a cycle:

    $ ./cuda29 -m 1514
    GeForce GTX 1080 Ti with 10GB @ 352 bits x 5505MHz
    Looking for 42-cycle on cuckoo30("",0) with 50% edges, 64*64 buckets, 1514 trims, and 64 thread blocks.
    Using 6976MB of global memory.
    nonce 0 k0 k1 k2 k3 a34c6a2bdaa03a14 d736650ae53eee9e 9a22f05e3bffed5e b8d55478fa3a606d
    Seeding completed in 552 + 303 ms
       4-cycle found
       2-cycle found
      16-cycle found
      40-cycle found
      26-cycle found
    findcycles edges 88 time 2 ms total 2217 ms
    0 total solutions

Note that 88 is exactly the sum length of all 5 cycles.
Insufficient trimming results in loss of  edges and possible loss of cycles:

    $ ./cuda29 -n 63 -m 128
    GeForce GTX 1080 Ti with 10GB @ 352 bits x 5505MHz
    Looking for 42-cycle on cuckoo30("",63) with 50% edges, 64*64 buckets, 128 trims, and 64 thread blocks.
    Using 6976MB of global memory.
    nonce 63 k0 k1 k2 k3 7d4f06d5f68dc772 331017080ac63322 e62926ee68af70ed cf2efe7e2f4dbc16
    Seeding completed in 546 + 302 ms
      42-cycle found
     192-cycle found
      84-cycle found
     320-cycle found
    findcycles edges 123505 time 266 ms total 2185 ms
    Solution 23ece 27e0856 2ad8c27 2cbb0b5 3694cdd 477a095 64de6fc 64e1c92 68e624d 6aa4c6f 6b1d0c2 76f07d2 c273122 c2e38ed c655cde c97ba17 e708130 ec8890d ecb9932 f28d66d f577aff 104d8441 116de91f 116e61cb 1178ea28 11840f8a 11ce10b0 12792630 12ae2388 140ae893 1439b9fd 146a3047 1538d93c 176cb068 17e01c9b 1876ee0a 1c871774 1d37d976 1d6fa785 1d9c1669 1d9d015e 1db85f7e
    Verified with cyclehash b06d3a638f4237c1d5d96fe57e549a83b76421522fb09af24d3520fb91f364f7
    1 total solutions

    $ ./cuda29 -n 63 -m 120
    GeForce GTX 1080 Ti with 10GB @ 352 bits x 5505MHz
    Looking for 42-cycle on cuckoo30("",63) with 50% edges, 64*64 buckets, 120 trims, and 64 thread blocks.
    Using 6976MB of global memory.
    nonce 63 k0 k1 k2 k3 7d4f06d5f68dc772 331017080ac63322 e62926ee68af70ed cf2efe7e2f4dbc16
    Seeding completed in 547 + 302 ms
    OOPS; losing 8821 edges beyond MAXEDGES=131072
    findcycles edges 131072 time 18 ms total 1936 ms
    0 total solutions

    $ ./cuda29 -m 120
    GeForce GTX 1080 Ti with 10GB @ 352 bits x 5505MHz
    Looking for 42-cycle on cuckoo30("",0) with 50% edges, 64*64 buckets, 120 trims, and 64 thread blocks.
    Using 6976MB of global memory.
    nonce 0 k0 k1 k2 k3 a34c6a2bdaa03a14 d736650ae53eee9e 9a22f05e3bffed5e b8d55478fa3a606d
    Seeding completed in 546 + 302 ms
    OOPS; losing 13696 edges beyond MAXEDGES=131072
       4-cycle found
       2-cycle found
      16-cycle found
    findcycles edges 131072 time 17 ms total 1970 ms
    0 total solutions

In this case the 13796 lost edges included one of the 42 cycle edges.
    
-n nonce
------------
The starting nonce. Default 0. This 32-bit number is stored in the final 4 bytes of the header,
just before hashing it with Blake2b to derive the siphash keys which determine the 2^30 node graph.

-r range
------------
The number of consecutive nonces to solve. Default 1. An actual miner would just keep incrementing
the nonce forever, until a solution that meets the difficulty threshold is found by any miner,
at which point a new header needs to be build.

-U seedAblocks -u seedAthreads
------------
The number of threadblocks and threads per block to use in seeding round A.
Since round 0, generating U nodes, is only writing buckets, it can use
arbitrarily many threadblocks. Here's a table of seedA times with various
combinations of blocks and threads per block (tpb):

tpb\blocks |  256 | 1024 | 4096 | 16384
---------- |  --- |  --- |  --- |  ----
       64  |   67 |   62 |   58 |    58
      128  |   45 |   45 |   54 |    41
      256  |   43 |   43 |   40 |    39
      512  |   56 |   46 |   45 |    45
     1024  |   58 |   59 |   57 |    60

There is a related compile time define -DFLUSHA=16 that should grow along with tpb to avoid excessive edge loss:

    $ ./cuda29 -n 63 -u 512
    GeForce GTX 1080 Ti with 10GB @ 352 bits x 5505MHz
    Looking for 42-cycle on cuckoo30("",63) with 50% edges, 64*64 buckets, 176 trims, and 64 thread blocks.
    Using 6976MB of global memory.
    nonce 63 k0 k1 k2 k3 7d4f06d5f68dc772 331017080ac63322 e62926ee68af70ed cf2efe7e2f4dbc16
    Seeding completed in 581 + 304 ms
      42-cycle found
     192-cycle found
      84-cycle found
     320-cycle found
    findcycles edges 67098 time 174 ms total 2176 ms
    Solution 23ece 27e0856 2ad8c27 2cbb0b5 3694cdd 477a095 64de6fc 64e1c92 68e624d 6aa4c6f 6b1d0c2 76f07d2 c273122 c2e38ed c655cde c97ba17 e708130 ec8890d ecb9932 f28d66d f577aff 104d8441 116de91f 116e61cb 1178ea28 11840f8a 11ce10b0 12792630 12ae2388 140ae893 1439b9fd 146a3047 1538d93c 176cb068 17e01c9b 1876ee0a 1c871774 1d37d976 1d6fa785 1d9c1669 1d9d015e 1db85f7e
    Verified with cyclehash b06d3a638f4237c1d5d96fe57e549a83b76421522fb09af24d3520fb91f364f7
    1 total solutions

    $ ./cuda29 -n 63 -u 1024
    GeForce GTX 1080 Ti with 10GB @ 352 bits x 5505MHz
    Looking for 42-cycle on cuckoo30("",63) with 50% edges, 64*64 buckets, 176 trims, and 64 thread blocks.
    Using 6976MB of global memory.
    nonce 63 k0 k1 k2 k3 7d4f06d5f68dc772 331017080ac63322 e62926ee68af70ed cf2efe7e2f4dbc16
    Seeding completed in 690 + 290 ms
    findcycles edges 4496 time 2 ms total 1991 ms
    0 total solutions

-v seedBthreads
------------
The number of threads per block to use in seeding round B.
Here's the effect on seedB times:

tpb |  16 | 32 | 64 | 128 | 256 | 512 | 1024
--- |  -- | -- | -- | --- | --- | --- |  ---
 ms |  93 | 55 | 36 |  31 |  33 |  34 |   32

Again there is a related compile time define -DFLUSHB=8 that should grow along with tpb to avoid excessive edge loss.

-w Trimthreads
------------
The number of threads per block to use in trimming rounds.
Here's the effect on total times:

tpb |  32 |  64 | 128 | 256 | 512 | 1024
--- | --- | --- | --- | --- | --- | ----
 ms | 614 | 389 | 284 | 239 | 228 |  233



The three remaining options below have only a negligible impact on runtimes.

-y threads
------------
The number of threads per block to use in bucket compaction.

-Z threads
------------
The number of thread blocks to use in edge index recovery.

-z threads
------------
The number of threads per block to use in edge recovery.

Happy tuning!
