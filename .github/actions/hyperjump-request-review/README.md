# hyperjump-request-review

This action is the backend fo the `request-review` action. It is intended to
receive requests from the hyperjump server, via the `repository_dispatch`
trigger, with the `request-review` type and request reviews from specific
users or teams.

## Usage

To use this, add a workflow file such as:

```yaml
name: (hyperjump) request-review

on:
  repository_dispatch:
    types: [request-review]

jobs:
  labels:
    runs-on: ubuntu-latest
    name: (hyperjump) request-review
    steps:
      - name: checkout
        uses: actions/checkout@v2
      - name: request-review
        uses: ./.github/actions/hyperjump-request-review
        with:
          github-token: ${{ secrets.HYPERJUMP_TOKEN }}
          number: ${{ github.event.client_payload.number }}
          users: ${{ join(github.event.client_payload.users) }}
          teams: ${{ join(github.event.client_payload.teams) }}
```

Note that you cannot use the default `GITHUB_TOKEN` because even write-scoped
`GITHUB_TOKEN`s do not have the ability to request review from a team.

## Updating

After making changes to `src/index.js` you must run `npm run prepare` to
generate `dist/*` files and check those in for the action to run successfully.
