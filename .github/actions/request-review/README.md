# request-review

This action requests reviewers for a pull request. The pull request to label
is implied by where the action is triggered.

## Usage

Here's an example workflow using this action:

```yaml
request-dep-audit-review:
    runs-on: ubuntu-latest
    name: request dep-audit team review
    steps:
      - name: checkout
        uses: actions/checkout@v2
      - name: request review
        uses: ./.github/actions/request-review
        with:
          teams: dep-audit
```

## Inputs

### `users`

Comma separated list of users to request reviews from. For example `metajack`
or `bmwill,davidiw`.

### `teams`

Comma separated list of teams to request review from.
