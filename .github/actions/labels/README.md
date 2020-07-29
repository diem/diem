# labels

This action adds and removes labels from an issue or pull request. The issue
or pull request to label is implied by where the action is triggered.

## Usage

Here's an example workflow using this action:

```yaml
label-prs:
    runs-on: ubuntu-latest
    name: label issue
    steps:
      - name: checkout
        uses: actions/checkout@v2
      - name: add testing label
        uses: ./.github/actions/labels
        with:
          add: testing
```

## Inputs

### `add`

Comma separated list of labels to add. For example `testing` as above or
`testing,needs-review`.

### `remove`

Comma separated list of labels to remove.
