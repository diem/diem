# pr-base-ref

This action gets the base ref for the pull request, which is the parent ref of
the first commit in the pull request. Note that this is needed as GitHub's
`pull_request.base.sha` is the latest ref of the target branch, which is often
not the parent of the first commit.

This action can be used if you need to calculate the delta of a PR comparing
its changes against the changes of the parent commit. If you use
`pull_request.base.sha` you will mix the PR's changes up with the changes that
also occurred on the target branch since the PR branch was created.

## Usage

Here's an example workflow using this action:

```yaml
  get-pr-base-ref:
    runs-on: ubuntu-latest
    name: checkout base ref
    steps:
      - name: get pr base ref
        id: pr-base-ref
        uses: ./.github/actions/pr-base-ref
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
      - name: checkout base ref
        uses: actions/checkout@v2
        with:
          ref: ${{ steps.pr-base-ref.outputs.ref }}
```

## Inputs

### `github-token`

For passing along the GitHub token.
