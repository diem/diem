# comment

This action adds a comment to an issue or pull request. The issue or pull
request is implied by where the action is triggered.

## Usage

Here's an example workflow using this action:

```yaml
  comment-on-issue:
    runs-on: ubuntu-latest
    name: comment on issue
    steps:
      - name: checkout
        uses: actions/checkout@v2
      - name: post comment
        uses: ./.github/actions/comment
        with:
          comment: "this is a comment"
```

## Inputs

### `comment`

The comment text.

### `tag`

A string tag that will be added to the hidden metadata of the
comment. Defaults to "unknown".

Since all comments come from the `github-actions` user, this allows workflows
to distinguish specific comments if needed, for example, when using
`delete-older`.

### `delete-older`

If set to true, older comments with the same tag will be deleted before a new
comment is posted. This can be used to make the workflow less chatty.
