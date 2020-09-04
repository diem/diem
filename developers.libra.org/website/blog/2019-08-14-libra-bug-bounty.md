---
author: Michael Engle, Libra Association
title: Libra Bug Bounty Open to All
---

<script>
	let items = document.getElementsByClassName("post-meta");	
	for (var i = items.length - 1; i >= 0; i--) {
		console.log(items[i], items[i].innerText);
		if (items[i].innerHTML = '<p class="post-meta">August 14, 2019</p>') items[i].innerHTML = '<p class="post-meta">August 27, 2019</p>';
	}
    var slug = location.pathname.slice(location.pathname.lastIndexOf('/')+1);
    var redirect = 'https://libra.org/en-US/blog/' + slug;
    window.location = redirect;
</script>

When we built the Libra Blockchain, security was top of mind. If people are going to rely on Libra for their everyday financial needs, it is critical that the infrastructure behind it be dependable and safe. This is one of the reasons we shared our plans well in advance of launch and open-sourced an early-stage version of the Libra Blockchain code, Libra Core, under an Apache 2.0 License. This testnet will help us gather feedback from the community about the direction of the project and work toward ensuring a scalable, reliable, and secure launch.

Another way to gather feedback and reinforce the security of the Libra Blockchain is through a bug bounty. **Today we are excited to announce the launch of the Libra Bug Bounty program, which is open to security researchers around the world.** You can find more information about the program [here](https://hackerone.com/libra).

We kicked off our bug bounty efforts as soon as we announced the plans for Libra, on June 18th, with a beta bug bounty program. We invited 50 security researchers with blockchain expertise and encouraged their deep scrutiny of the platform. This helped us fine-tune and tweak the program before opening it up more broadly.

Now that the program is open to the public, we hope to further accelerate and expand this feedback loop. Our rewards program is designed to encourage members of the security community to dig deep, helping us find even the most subtle bugs. We want to help our researchers uncover issues while the Libra Blockchain is still in testnet and no real money is in circulation. Participants can receive up to $10,000 in rewards for discovering the most critical issues.

Bug bounties are a well-established way to encourage members of the security community to detect and report security vulnerabilities in exchange for potential rewards. And it’s not just for blockchains. Many companies across industries have bug bounty programs of their own to help encourage developers to report security issues with their respective technologies.

With the launch of the Libra Bug Bounty, we are excited to build an open and vibrant network of security and privacy researchers around the globe. We know it will take a global community to launch a global cryptocurrency, and we are committed to taking the time to get this right.

For more information about the Libra Bug Bounty program, visit [https://hackerone.com/libra](https://hackerone.com/libra).
