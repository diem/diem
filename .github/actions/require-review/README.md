# require-review

This action requires review from specific users in order to succeed. The pull
request is implied by where the action is triggered.

## Usage

Here's an example workflow using this action:

```yaml
require-dep-audit-review:
    runs-on: ubuntu-latest
    name: require dep-audit team review
    steps:
      - name: checkout
        uses: actions/checkout@v2
      - name: require review
        uses: ./.github/actions/require-review
        with:
          users: davidiw,mimoo
```

## Inputs

### `users`

Comma separated list of users to require reviews from. For example `metajack`
or `bmwill,davidiw`.
