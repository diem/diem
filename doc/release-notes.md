*After branching off for a major version release of Bitcoin Core, use this
template to create the initial release notes draft.*

*The release notes draft is a temporary file that can be added to by anyone. See
[/doc/developer-notes.md#release-notes](/doc/developer-notes.md#release-notes)
for the process.*

*Create the draft, named* "*version* Release Notes Draft"
*(e.g. "0.20.0 Release Notes Draft"), as a collaborative wiki in:*

https://github.com/bitcoin-core/bitcoin-devwiki/wiki/

*Before the final release, move the notes back to this git repository.*

*version* Release Notes Draft
===============================

Bitcoin Core version *version* is now available from:

  <https://bitcoincore.org/bin/bitcoin-core-*version*/>

This is a new major version release, including new features, various bugfixes
and performance improvements, as well as updated translations.

Please report bugs using the issue tracker at GitHub:

  <https://github.com/bitcoin/bitcoin/issues>

To receive security and update notifications, please subscribe to:

  <https://bitcoincore.org/en/list/announcements/join/>

How to Upgrade
==============

If you are running an older version, shut it down. Wait until it has completely
shut down (which might take a few minutes for older versions), then run the
installer (on Windows) or just copy over `/Applications/Bitcoin-Qt` (on Mac)
or `bitcoind`/`bitcoin-qt` (on Linux).

Upgrading directly from a version of Bitcoin Core that has reached its EOL is
possible, but might take some time if the datadir needs to be migrated.  Old
wallet versions of Bitcoin Core are generally supported.

Downgrading warning
-------------------

The chainstate database for this release is not compatible with previous
releases, so if you run 0.15 and then decide to switch back to any
older version, you will need to run the old release with the `-reindex-chainstate`
option to rebuild the chainstate data structures in the old format.

If your node has pruning enabled, this will entail re-downloading and
processing the entire blockchain.

Compatibility
==============

Bitcoin Core is supported and extensively tested on operating systems using
the Linux kernel, macOS 10.10+, and Windows 7 and newer.  It is not recommended
to use Bitcoin Core on unsupported systems.

Bitcoin Core should also work on most other Unix-like systems but is not
frequently tested on them.

From 0.17.0 onwards, macOS <10.10 is no longer supported.  0.17.0 is
built using Qt 5.9.x, which doesn't support versions of macOS older than
10.10.  Additionally, Bitcoin Core does not yet change appearance when
macOS "dark mode" is activated.

In addition to previously-supported CPU platforms, this release's
pre-compiled distribution also provides binaries for the RISC-V
platform.

Notable changes
===============

New RPCs
--------

- `getbalances` returns an object with all balances (`mine`,
  `untrusted_pending` and `immature`). Please refer to the RPC help of
  `getbalances` for details. The new RPC is intended to replace
  `getunconfirmedbalance` and the balance fields in `getwalletinfo`, as well as
  `getbalance`. The old calls may be removed in a future version.

Updated RPCs
------------

Note: some low-level RPC changes mainly useful for testing are described in the
Low-level Changes section below.

- The `sendmany` RPC had an argument `minconf` that was not well specified and
  would lead to RPC errors even when the wallet's coin selection would succeed.
  The `sendtoaddress` RPC never had this check, so to normalize the behavior,
  `minconf` is now ignored in `sendmany`. If the coin selection does not
  succeed due to missing coins, it will still throw an RPC error. Be reminded
  that coin selection is influenced by the `-spendzeroconfchange`,
  `-limitancestorcount`, `-limitdescendantcount` and `-walletrejectlongchains`
  command line arguments.


Low-level changes
=================

Configuration
------------

- An error is issued where previously a warning was issued when a setting in
  the config file was specified in the default section, but not overridden for
  the selected network. This change takes only effect if the selected network
  is not mainnet.

Network
-------

- When fetching a transaction announced by multiple peers, previous versions of
  Bitcoin Core would sequentially attempt to download the transaction from each
  announcing peer until the transaction is received, in the order that those
  peers' announcements were received.  In this release, the download logic has
  changed to randomize the fetch order across peers and to prefer sending
  download requests to outbound peers over inbound peers. This fixes an issue
  where inbound peers can prevent a node from getting a transaction.

Wallet
------

- When in pruned mode, a rescan that was triggered by an `importwallet`,
  `importpubkey`, `importaddress`, or `importprivkey` RPC will only fail when
  blocks have been pruned. Previously it would fail when `-prune` has been set.
  This change allows to set `-prune` to a high value (e.g. the disk size) and
  the calls to any of the import RPCs would fail when the first block is
  pruned.

Credits
=======

Thanks to everyone who directly contributed to this release:


As well as everyone that helped translating on [Transifex](https://www.transifex.com/bitcoin/bitcoin/).
