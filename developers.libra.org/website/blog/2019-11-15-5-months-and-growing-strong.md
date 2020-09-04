---
author: Michael Engle, Libra Association
title: Five months and growing strong: the Libra project
---

<script>
    let items = document.getElementsByClassName("post-meta");   
    for (var i = items.length - 1; i >= 0; i--) {
        console.log(items[i], items[i].innerText);
        if (items[i].innerHTML = '<p class="post-meta">November 15, 2019</p>') items[i].innerHTML = '<p class="post-meta">November 15, 2019</p>';
    }
    var slug = location.pathname.slice(location.pathname.lastIndexOf('/')+1);
    var redirect = 'https://libra.org/en-US/blog/' + slug;
    window.location = redirect;    
</script>

It's been just five months since we announced the Libra project on June 18, 2019, and nearly a month since the Libra Association charter was signed by its members in Geneva. We've been working diligently to build the global community of developers and the technical infrastructure needed to support it. Here are a few highlights of what we've been working on:

- Inviting community participation in shaping how we get to mainnet by making a detailed technical roadmap available to all. 
- Encouraging developers to test Libra network functionality by launching and continuously improving testnet, which has logged more than 51,000 transactions since we reset the testnet on September 17, 2019. 
- Simplifying how developers work with the Libra project team, so they can focus more on building — by integrating our online developer community, GitHub, CLA process, and more.
- Teaching developers of all backgrounds and skill levels how to work with the Libra network by consistently publishing documentation and technical blogs.
- Incentivizing developers to help us find and fix bugs faster by launching a Bug Bounty program.

Our community of developers has responded enthusiastically, logging an amazing 34 projects in just seven weeks since launch of the testnet:

- 10 wallets
- 11 blockchain explorers
- 2 IDEs
- 1 API
- 11 clients

We want to say a very big thank you to all the developers in the community for all the hard work. We’re thrilled to be part of such a passionate community!


## Building involvement in Libra Core

The success of the Libra project lies with the extended community that supports it. To help us build on our existing foundation by getting even more people involved, we recently held a Libra Core Summit with Libra Association technical team members. It was our first step towards a collaborative development plan for Libra Core and Move. In the future, we intend to continue to host events for all developers to discuss issues, challenges, and opportunities in the Libra ecosystem. 

In addition to an expanded overview of the backstory, economics, and vision for the Libra project, topics covered include:

- An overview of Libra Core and its roadmap
- How to run a Libra node
- How to build a Libra wallet
- How to scale the Libra network
- Libra wallet interoperability


## Expanding to the pre-mainnet

Until we launch mainnet, the best and fastest way we have to demonstrate Libra network functionality and provide early access is through our pre-mainnet. Pre-mainnet, a version of testnet available to Libra Association members, makes it easier and faster to test, troubleshoot, diagnose, and resolve software edge cases. One measure of our success in this phase is the number of nodes that are live on the pre-mainnet. Currently, pre-mainnet has:

- Deployed nodes: 7
- In-process nodes: 6
- In-process nodes without a technical team: 8

For those organizations without a technical team to implement a node, the Libra Association is working on a strategy to support deployment in 2020, when the Libra Core feature set is complete.

The Libra Association intends to deploy 100 nodes on the mainnet, representing a mix of on-premises and cloud-hosted infrastructure. All the work we are doing now on pre-mainnet supports this goal of greater resiliency on the network through a wider diversity of infrastructure.


## Deepening the developer experience

As a Libra developer, we know that you commit significant time and resources to your work. We want to give you tools that will support you in making the most of your time and technical investments. Our work here includes:

- Providing guides, documentation, and content for blockchain, smart contract, and wallet/client developers
- Providing structured discussions and support
- Advocating for transparency and access
- Scaling outreach through tooling, events, programs, and bounties
- Supporting engineering efforts of the Technical Steering Committee and Roadmap process.

Now, we want to make it easier and faster to submit code and documentation to the Libra project. On November 26th, we’re launching a new, streamlined process for completing Contributor License Agreements (CLAs). As with many open-source projects, you must complete and sign a CLA before you can submit code to the Libra project on GitHub.

Our new process streamlines how CLAs are submitted, reviewed, and verified, whether for individuals or for contributors working on behalf of a corporation or business.

You can start the process from GitHub. Choose which type of CLA best fits your situation (individual or corporate), fill out a form, and go from there. If your CLA requires additional signatures, we’ll make sure to collect those so you can focus on your code.

As in any open source project, a CLA qualifies contributions for review, but submissions are not guaranteed to be accepted by maintainers. If you have an ambitious feature that you wish to add to Libra Core, and which does not fit into the existing Roadmap, reach out through engineering channels or to the Libra Association to start a discussion. Code review and code acceptance is managed by project maintainers. Review the [Contribution Guide](https://developers.libra.org/docs/community/contributing) and [Coding Guidelines](https://developers.libra.org/docs/community/coding-guidelines) prior to submitting your changes.


## Expanding the technical Roadmap and technical governance 

You can check the technical progress we're making toward the Libra Core mainnet launch on our [Roadmap](https://github.com/orgs/libra/projects/1). Completed work includes:

- Libra Canonical Serialization (LCS)
- MVP for full nodes
- MVP for vectors in Move
- Events

We have a lot of work still to do, and we need a passionate, dedicated community to help us all get to mainnet launch. To help us get there and make the best use of all the knowledge in our community, we plan to launch a Technical Steering Committee (TSC) in the coming months. 

The TSC will oversee and coordinate the technical design and development of the Libra network on behalf of the Libra Association members. Libra Association members will decide together how the TSC will operate, including how its members will be determined, the full scope of its responsibilities, and how the extended community can support specific initiatives. 


## What opportunities interest you as a Libra developer?

As a Libra developer, you’re an incredibly important contributor to the Libra project. We’re excited to be on this journey with you, and with everyone in our extended developer community, and we look forward to our continued progress toward mainnet launch.

We invite you to dive in deeper, wherever you see the best opportunity to help us all move forward faster: 

- [Contribute code](https://github.com/libra/libra)
- Share your questions and answers in the [Libra developer community page](https://community.libra.org/)
- Read the [Libra documentation](https://developers.libra.org/docs/welcome-to-libra) and [blog](https://developers.libra.org/blog/)
- [Share your projects](https://community.libra.org/c/Please-follow-this-category-for-projects-made-on-the-Libra-testnet)
- [Follow us on Twitter](https://twitter.com/libradev) to stay up with the latest
