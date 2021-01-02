# Docker Hub Login and Image Signing #

## What ##

Calculates the changes being tested by this build and sets fout environment variables in to the environment to be used by subsequent steps.

**PULL_REQUEST** The PR number if this is a pull request, of a bor's build of a landing pr.
**BASE_GITHASH** If a PR the shared base revision of the target branch.  If a push, the last successful build in github actions.  See below.
**CHANGED_FILE_OUTPUTFILE** The name of a temp file containing all changed files since the base revision of this build.
**TARGET_BRANCH** If a PR, or a bors build, the branch targeted for this pull request.

This actions handles three different case.

### Case 1:  A pull request ###

Calculates all file changes since the base git revision shared between this pr (which may have many commits)
and the target branch (which may have moved on)

### Case 2:  A Push to a long lived branch (main, master, etc...) ###

Uses the branch name as supplied by **GITHUB_REF** to find the **BRANCH** name, and then calls the github api to find
the last successfully completed build of this branch, for the supplied **workflow-file**.  This is used find a base githash.

```${GITHUB_API_URL}/repos/${GITHUB_REPOSITORY}/actions/workflows/${WORKFLOW_FILE}/runs?branch=${BRANCH}&status=completed&event=push```

If that base githash is a parent of this build (not rewritten out of history) it will be used to as the **BASE_GITHASH**.
If the git checkout doesn't contain history (the default behavior for the checkout github action ) or the base githash has been written
out of history the base githash will remain unset.

### Case 3: A bors build of a pull request ###

Will look at the current commit message to extract the bors supplied **PR_NUMBER** in the second to last line of the commit message.
That **PR_NUMBER** is used to look up the PR's target branch via the github API.

```https://api.github.com/repos/${GITHUB_SLUG}/pulls/$PR_NUMBER"```
## Usage ##

Example Usage:

``` yaml
TODO:
```

## Parameters ##

### workflow-file ###

Should be the name of the github action workflow file found in .github/workflows/**&lt;filename&gt;**. Including the `.yml` extension.
Used to look up prior builds.   Sadly there is no reliable way to determine the workflow file name from the user defined `name:` attribute
in the workflow file as supplied by the user.

```
   Unfortunately,  https://github.community/t/access-workflow-filename-even-when-name-is-defined/17641 is not a viable solution as multiple jobs
   can use the same name.
```
