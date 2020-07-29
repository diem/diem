# hyperjump-labels

This is the backend for the `labels` action. It is intended to receive
requests from the hyperjump server, via the `repository_dispatch` trigger,
with the `labels` type and add and remove the requested labels from a
specified issue or PR.

This should not be used directly in normal workflows.

## Usage

To use this, add a workflow file such as:

```yaml
name: (hyperjump) labels

on:
  repository_dispatch:
    types: [labels]

jobs:
  labels:
    runs-on: ubuntu-latest
    name: (hyperjump) labels
    steps:
      - name: checkout
        uses: actions/checkout@v2
      - name: labels
        uses: ./.github/actions/hyperjump-labels
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
          number: ${{ github.event.client_payload.number }}
          add: ${{ join(github.event.client_payload.add) }}
          remove: ${{ join(github.event.client_payload.remove) }}
```

## Updating

After making changes to `src/index.js` you must run `npm run prepare` to
generate `dist/*` files and check those in for the action to run successfully.
