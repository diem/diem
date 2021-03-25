---
id: assets-proof
title: Proofs of Assets
custom_edit_url: https://github.com/diem/diem/edit/main/client/assets-proof/README.md
---

One of the most needed auditing functionalities for financial solvency and tax reporting purposes is to prove ownership
of blockchain reserves, a process known as Proof of Assets (PoA).
The assets-proof CLI component hosts functionality required to return (and in the future: to prove) total on-chain
assets owned by Diem VASPs.

## Overview of Soft PoA

Generally, a PoA in Diem implies showing that a parent VASP account is in possession of assets of some specific
currency(ies) value. However, there is a subtle distinction on how to actually show this. *Soft PoA* uses Diem's
hierarchical address structure to sum the values of the parent VASP and all of its child accounts at a particular
timestamp (or version). The output is very similar to an accounting sheet of current on-chain balances per parent and
children accounts.

While straightforward, this proof does not provide key possession guarantees at the time of the
audit taking place. For instance, account holders might have lost access to their keys, which would make them unable
to spend their assets. On the other hand, this process is non-interactive, allowing for easier and more frequent audits.
We highlight that unlike other blockchains, this is possible in Diem due to its hierarchical identity-address binding
which makes collusion more difficult and traceable; a *WalletA* cannot just temporarily borrow its private key to a
*WalletB* (an on-chain transaction should happen posing the risk of being censored).

## Example usage
```shell script
# Retrieve proof of assets statement for the parent VASP ed34ba9396da470da8de0945555549f8 and its children.
$ cargo run -p diem-assets-proof -- collect \
    --json-server 'https://testnet.diem.com/v1' \
    --parent-vasp 'ed34ba9396da470da8de0945555549f8' \
    --child-vasps 'f236d4d977cbf08fe6e8e796bb6b41bd' \
    --child-vasps 'ad8c1f9be3b9de371567eda247c1f290' \
    --child-vasps '228d25d0c232c010be8bc1613f833ea0'

{
  "diem_chain_id": 2,
  "diem_ledger_version": 88497519,
  "diem_ledger_timestampusec": 1616457122815922,
  "accumulator_root_hash": "75e03d04f13f15450807ced2d8707fd90b3a95d71ce981396deacf6134f20b64",
  "all_child_vasps_valid": true,
  "total_unfrozen_balances": {
    "XUS": 100000000000
  },
  "currencies": {
    "XDX": {
      "scaling_factor": 1000000,
      "fractional_part": 1000,
      "to_xdx_exchange_rate": 1.0
    },
    "XUS": {
      "scaling_factor": 1000000,
      "fractional_part": 100,
      "to_xdx_exchange_rate": 1.0
    }
  },
  "parent_vasp": {
    "address": "ED34BA9396DA470DA8DE0945555549F8",
    "balances": {
      "XUS": 70000000000
    },
    "human_name": "No. 63291 VASP",
    "base_url": "http://localhost:54330",
    "num_children": 3
  },
  "child_vasps": {
    "228D25D0C232C010BE8BC1613F833EA0": {
      "result": {
        "balances": {
          "XUS": 10000000000
        }
      }
    },
    "AD8C1F9BE3B9DE371567EDA247C1F290": {
      "result": {
        "balances": {
          "XUS": 10000000000
        }
      }
    },
    "F236D4D977CBF08FE6E8E796BB6B41BD": {
      "result": {
        "balances": {
          "XUS": 10000000000
        }
      }
    }
  }
}
```
