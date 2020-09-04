---
author: Libra Engineering Team
title: Libra Core Roadmap #3
---

<script>
    let items = document.getElementsByClassName("post-meta");   
    for (var i = items.length - 1; i >= 0; i--) {
        if (items[i].innerHTML = '<p class="post-meta">February 28, 2020</p>') items[i].innerHTML = '<p class="post-meta">February 28, 2020</p>';
    }
    var slug = location.pathname.slice(location.pathname.lastIndexOf('/')+1);
    var redirect = 'https://libra.org/en-US/blog/' + slug;
    window.location = redirect;    
</script>

## Roadmap #2 retrospective

### Overview

In [Roadmap #2](https://developers.libra.org/blog/2019/12/17/libra-core-roadmap-2), we made solid progress on improvements to Move. We iterated on the design of the Move source language and cleaned up key parts of the VM. Further improvements, including additional VM cleanup, design of the Libra Upgrade protocol, and VM versioning, are still in progress.

We made incremental progress laying the groundwork toward a more open programming model in the Move language.

### Delivered

- Core Move modules
  - Validator reconfiguration: implemented VM and core module support
- Move VM
  - Completed cleanup of the VM interpreter
  - Revised the VM architecture to separate Move and Libra protocol layers, and started on the implementation of this change
  - Implemented the new borrow-checker algorithm
  - Continued progress on the design of the Libra Upgrade protocol and VM versioning
  - Finished incremental changes to Events in response to pub/sub feedback
- Tooling
  - Updated tooling for formal verification to support new language features
  - Added support for specifications in the Move IR compiler
  - Pre-released code for [guppy: a library for querying Rust dependency graphs](https://github.com/calibra/cargo-guppy), and cargo-guppy, a command-line frontend for the guppy library
  - Added support for generic types to automate bytecode test generation
  - Enhanced the functional test framework with significant usability improvements
- Move language
  - Conducted design reviews of source language
  - Made significant language changes to address problems in the initial implementation
  - Added source language tests
  - Defined new core types: added new u8 and u128 integer types, along with related shift and cast operations; removed string type
- Communications
  - Submitted a paper on Move to PLDI

## Roadmap #3: What's next in Q1

For the first sprint of 2020, we are focusing on essential tasks in our roadmap to mainnet launch. We have also identified that one of the most urgent developer needs is at the client level, in particular, helping organizations launch consumer-facing wallets. We've kicked off a separate effort to address the conceptual and technical hurdles that wallet developers face today.

**1) Stabilize Move language for mainnet**

Next steps for the Move language include closing and executing on design decisions for the source language, including:

- Finalizing core types
- Cutting over to source language compiler
- Executing smaller account addresses
- Finalizing access paths

**2) Complete custodial modules required for mainnet**

The remaining non-currency modules will support custody of funds — particularly, leveraging the new LibraTime module for withdrawal limitations and transaction time to live (TTL). This work effort includes:

- LibraTime module
- Multisig accounts
- Key rotation capability
- Cold wallet functionality

**3) Advance formal verification tooling and integrate with CI**

Before the code completion milestone, we would like to have the tooling in place to run formal verification over our core modules and extend the scope of verification. However, formal specification will take longer than we anticipated and will likely result in some changes to the design and implementation of our modules.

- Formal verification tooling updates
- Expand the pre/post condition specification of core modules

**4) VM cleanup and infrastructure improvements**

The roadmap includes continuing code cleanup, testing, and operational awareness improvements. This will allow us to validate key assumptions, run realistic tests, and ready the network for launch. This work effort includes:

- Complete VM loader cleanup
- Complete VM file format cleanup
- VM versioning implementation
- Basic debugging support
- Libra Upgrade protocol

**5) Collaborate with documentation engineering team to produce developer content**

To address developer interest in Move and the Libra project, we will share additional content about key considerations for client developers and conceptual overviews of the system. This content focus aligns to our wallet developer effort and content requests we have received from the Libra Developer Community. Topics we will cover include wallets, wallet types, and custody.

## Track our progress

Be sure to follow our progress on GitHub, where we've posted a [Kanban board](https://github.com/orgs/libra/projects/1#card-28078342) of roadmap projects.
