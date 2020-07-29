# hyperjump-comment

This is the backend for the `comment` action. It is intended to receive
requests from the hyperjump server, via the `repository_dispatch` trigger,
with the `comment` type and add a comment to an issue or pull request.

This should not be used directly in normal workflows.

## Usage

To use this, add a workflow file such as:

```yaml
name: (hyperjump) comment

on:
  repository_dispatch:
    types: [comment]

jobs:
  comment:
    runs-on: ubuntu-latest
    name: (hyperjump) comment
    steps:
      - name: checkout
        uses: actions/checkout@v2
      - name: comment
        uses: ./.github/actions/hyperjump-comment
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
          number: ${{ github.event.client_payload.number }}
          comment: ${{ github.event.client_payload.comment }}
```

## Updating

After making changs to `src/index.js` you must run `npm run prepare` to
generate `dist/*` and check those in for the action to run successfully.
