# Contributing to Libra

Our goal is to make contributing to the Libra project easy and transparent.

> **Note**: As the Libra Core project is currently an early-stage prototype, it
> is undergoing rapid development. While we welcome contributions, before
> making substantial contributions be sure to discuss them in the Discourse
> forum to ensure that they fit into the project roadmap.

## On Contributing

### Libra Core

To contribute to the Libra Core implementation, first start with the proper
development copy.

To get the development installation with all the necessary dependencies for
linting, testing, and building the documentation, run the following:
```bash
git clone https://github.com/libra/libra.git
cd libra
./scripts/dev_setup.sh
cargo build
cargo xtest
```

## Our Development Process

#### Code Style, Hints, and Testing

Refer to our [Coding
Guidelines](https://developers.libra.org/docs/community/coding-guidelines) for
detailed guidance about how to contribute to the project.

#### Documentation

Libra's website is also open source (the code can be found in this
[repository](https://github.com/libra/website/)).  It is built using
[Docusaurus](https://docusaurus.io/):

If you know Markdown, you can already contribute! This lives in the [website
repo](https://github.com/libra/website).

## Developer Workflow

Changes to the project are proposed through pull requests. The general pull
request workflow is as follows:

1. Fork the repo and create a topic branch off of `master`.
2. If you have added code that should be tested, add unit tests.
3. If you have changed APIs, update the documentation. Make sure the
   documentation builds.
4. Ensure all tests and lints pass on each and every commit that is part of
   your pull request.
5. If you haven't already, complete the Contributor License Agreement (CLA).
6. Submit your pull request.

## Authoring Clean Commits

#### Logically Separate Commits

Commits should be
[atomic](https://en.wikipedia.org/wiki/Atomic_commit#Atomic_commit_convention)
and broken down into logically separate changes. Diffs should also be made easy
for reviewers to read and review so formatting fixes or code moves should not
be included in commits with actual code changes.

#### Meaningful Commit Messages

Commit messages are important and incredibly helpful for others when they dig
through the commit history in order to understand why a particular change
was made and what problem it was intending to solve. For this reason commit
messages should be well written and conform with the following format:

All commit messages should begin with a single short (50 character max) line
summarizing the change and should skip the full stop. This is the title of the
commit. It is also preferred that this summary be prefixed with "[area]" where
the area is an identifier for the general area of the code being modified, e.g.

```
* [ci] enforce whitelist of nightly features
* [language] removing VerificationPass trait
```

A non-exhaustive list of some other areas include:
* consensus
* mempool
* network
* storage
* execution
* vm

Following the commit title (unless it alone is self-explanatory), there should
be a single blank line followed by the commit body which includes more
detailed, explanatory text as separate paragraph(s). It is recommended that the
commit body be wrapped at 72 characters so that Git has plenty of room to
indent the text while still keeping everything under 80 characters overall.

The commit body should provide a meaningful commit message, which:
* Explains the problem the change tries to solve, i.e. what is wrong
  with the current code without the change.
* Justifies the way the change solves the problem, i.e. why the
  result with the change is better.
* Alternative solutions considered but discarded, if any.

#### References in Commit Messages

If you want to reference a previous commit in the history of the project, use
the format "abbreviated sha1 (subject, date)", with the subject enclosed in a
pair of double-quotes, like this:

```bash
Commit 895b53510 ("[consensus] remove slice_patterns feature", 2019-07-18)
noticed that ...
```

This invocation of `git show` can be used to obtain this format:

```bash
git show -s --date=short --pretty='format:%h ("%s", %ad)' <commit>
```

If a commit references an issue please add a reference to the body of your
commit message, e.g. `issue #1234` or `fixes #456`. Using keywords like
`fixes`, `resolves`, or `closes` will cause the corresponding issue to be
closed when the pull request is merged.

Avoid adding any `@` mentions to commit messages, instead add them to the PR
cover letter.

## Responding to Reviewer Feedback

During the review process a reviewer may ask you to make changes to your pull
request. If a particular commit needs to be changed, that commit should be
amended directly. Changes in response to a review *should not* be made in
separate commits on top of your PR unless it logically makes sense to have
separate, distinct commits for those changes. This helps keep the commit
history clean.

If your pull request is out-of-date and needs to be updated because `master`
has advanced, you should rebase your branch on top of the latest master by
doing the following:

```bash
git fetch upstream
git checkout topic
git rebase -i upstream/master
```

You *should not* update your branch by merging the latest master into your
branch. Merge commits included in PRs tend to make it more difficult for the
reviewer to understand the change being made, especially if the merge wasn't
clean and needed conflicts to be resolved. As such, PRs with merge commits will
be rejected.

## Bisect-able History

It is important that the project history is bisect-able so that when
regressions are identified we can easily use `git bisect` to be able to
pin-point the exact commit which introduced the regression. This requires that
every commit is able to be built and passes all lints and tests. So if your
pull request includes multiple commits be sure that each and every commit is
able to be built and passes all checks performed by CI.

## Contributor License Agreement

For pull request to be accepted by any Libra projects, a CLA must be signed.
You will only need to do this once to work on any of Libra's open source
projects. Individuals contributing on their own behalf can sign the [Individual
CLA](https://github.com/libra/libra/blob/master/contributing/individual-cla.pdf).
If you are contributing on behalf of your employer, please ask them to sign the
[Corporate
CLA](https://github.com/libra/libra/blob/master/contributing/corporate-cla.pdf).

## Issues

Libra uses [GitHub issues](https://github.com/libra/libra/issues) to track
bugs. Please include necessary information and instructions to reproduce your
issue.
