---
author: Eric Nakagawa, Calibra
title: How to contribute code to the Libra project: about the CLA process
---

<script>
    let items = document.getElementsByClassName("post-meta");   
    for (var i = items.length - 1; i >= 0; i--) {
        console.log(items[i], items[i].innerText);
        if (items[i].innerHTML = '<p class="post-meta">December 10, 2019</p>') items[i].innerHTML = '<p class="post-meta">December 10, 2019</p>';
    }
    var slug = location.pathname.slice(location.pathname.lastIndexOf('/')+1);
    var redirect = 'https://libra.org/en-US/blog/' + slug;
    window.location = redirect;    
</script>

Built on open source software, the Libra network relies on a vibrant community of developers to enhance and continue growing the system. Any consumer, developer, or business can use the Libra network, build products on top of it, and add value through their services. This is foundational to the goal of building more inclusive financial options for billions of people around the world.

Developers must complete and sign a CLA before submitting code to [the Libra project on GitHub](https://github.com/libra/libra). Use of a CLA is a common and well-accepted practice in open source projects, including well-known projects from Apache Software Foundation, Google (including Go), Python, Django, and more. CLAs can help remove ambiguities around a person or corporation's contributions and allow developers to contribute to and more confidently adopt projects.

## Submit a CLA for your contribution

To contribute, you must first submit a CLA. Follow these steps:

1. Go to [https://github.com/libra/libra](https://github.com/libra/libra).

2. Click **New pull request**.

3. Click **Create pull request**. If you've not submitted a pull request for Libra from your GitHub account before, you'll see this:

	![](https://libra.org/en-US/wp-content/uploads/sites/23/2019/12/CLA-blog-image-2.png)

4. Click **through to sign the CLA.**

5. Once you're on the [&quot;Sign an agreement&quot; page](https://libra.org/en-US/cla-sign/), choose which type of CLA best fits how you plan to contribute to the project (as an individual or as a member of a corporation).

	![](https://libra.org/en-US/wp-content/uploads/sites/23/2019/12/CLA-blog-image-3.png)

6. Follow the directions on screen to either sign and complete the Individual Agreement, or submit a request for a Corporate Agreement.

	**Individual**

	[Fill out the contribution form](https://libra.org/en-US/cla-sign-individual/). If you are an individual, you will be sent to GitHub to authorize your account. Once you accept, your request will be processed automatically.

	**On behalf of a corporation**

	[For contributions on behalf of a corporation,](https://libra.org/en-US/cla-sign-corporation/) once your form is submitted, a Libra project team member will review and reach out to complete the process of adding you and any other contributors from your organization. This process includes signature of a corporate CLA via DocuSign and then whitelisting members of your organization.


7. After CLA approval, you'll see a Signed icon and you'll be able to merge PRs.

	![](https://libra.org/en-US/wp-content/uploads/sites/23/2019/12/CLA-blog-image-4.png)

	As in any open source project, a CLA qualifies contributions for review, but submissions are not guaranteed to be accepted by the maintainers. Code review and code acceptance are managed by project maintainers. Review the [Contribution Guide](https://developers.libra.org/docs/community/contributing) and [Coding Guidelines](https://developers.libra.org/docs/community/coding-guidelines) prior to submitting your changes.

## Checking the status of your CLA

### Individual CLA
Checking your individual CLA status is easy. In each PR in GitHub, select "Recheck CLA". This will prompt the CLA verification process to run again.

If you haven't signed the CLA, you can click on the "Needs Signature" from your PR to be taken to the Individual CLA selection page. Once you've signed, you can recheck your CLA.

### Corporate CLA
The best way to check on your corporate CLA status is to simply email [support@libra.org](mailto:support@libra.org). Someone will be in contact with you within a few days to update the status of your CLA.

You can also recheck the status of your corporate CLA using these steps:

1. Complete the [Corporate CLA form.](https://libra.org/en-US/cla-sign-corporation/)
2. Select a Libra project member contact as the point of contact.
3. Wait for the authorized signer to sign the CLA.
4. When you receive the appropriate GitHub usernames and emails you can send them to [support@libra.org](mailto:support@libra.org) to have them added to the whitelist.

_Please note, this process can take several days, depending on the response time between each step._
