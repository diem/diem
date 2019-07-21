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
cargo test
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

## Pull Requests

During the initial phase of heavy development, we plan to only audit and review
pull requests. As the codebase stabilizes, we will be better able to accept
pull requests from the community.

1. Fork the repo and create your branch from `master`.
2. If you have added code that should be tested, add unit tests.
3. If you have changed APIs, update the documentation. Make sure the
   documentation builds.
4. Ensure the test suite passes.
5. Make sure your code passes both linters.
6. If you haven't already, complete the Contributor License Agreement (CLA).
7. Submit your pull request.

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
