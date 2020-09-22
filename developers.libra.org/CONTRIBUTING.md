# Contribution Guide

Our goal is to make contributing to the Libra project easy and transparent.

<blockquote class="block_note">
The Libra Core project is currently an early-stage prototype, it is undergoing rapid development. Before making any substantial contribution to the project, be sure to discuss it in the Discourse forum to ensure that it fits into the project roadmap.
</blockquote>

## Contributing to Libra Core

To contribute to Libra Core, ensure that you have the latest version of the codebase. To setup Libra Core with all the necessary dependencies for linting, testing, and building the documentation, run the following:
```bash
$ git clone https://github.com/libra/libra.git
$ cd libra
$ cargo install
$ cargo test
```

## Coding Guidelines

For detailed guidance on how to contribute to the Libra Core codebase refer to [Coding Guidelines](https://developers.libra.org/docs/coding-guidelines).

## Documentation

All developer documentation is published on the Libra developer site. The developer site is open source, and the code for building the site is located in this [repository](https://github.com/libra/libra/developers.libra.org). The developer site is built using [Docusaurus](https://docusaurus.io/).

If you are familiar with Markdown, you can start contributing!

## Pull Requests

During the initial phase of development, we plan to only audit and review the pull requests. As the codebase stabilizes, we will be able to accept pull requests from the community.

To submit your pull request:

1. Fork the `libra` repo and create your branch from `master`.
2. If you have added code that should be tested, add unit tests.
3. If you have made changes to APIs, update the relevant documentation, and build and test the developer site.
4. Verify and ensure that the test suite passes.
5. Make sure your code passes both linters.
6. Complete the Contributor License Agreement (CLA), if you haven't already done so.
7. Submit your pull request.

## Contributor License Agreement

For your pull requests to be accepted by any Libra project, you will need to sign a CLA. You will need to do this only once to work on any Libra open source project. Individuals contributing on their own behalf can sign the [Individual CLA](https://github.com/libra/libra/blob/master/contributing/individual-cla.pdf). If you are contributing on behalf of your employer, please ask them to sign the [Corporate CLA](https://github.com/libra/libra/blob/master/contributing/corporate-cla.pdf).

## Code of Conduct
Please refer to the [Code of Conduct](https://github.com/libra/libra/blob/master/CODE_OF_CONDUCT.md) for guidelines on interacting with the community.

## Issues

Libra uses [GitHub issues](https://github.com/libra/libra/issues) to track bugs. Please include necessary information and instructions to reproduce your issue.
