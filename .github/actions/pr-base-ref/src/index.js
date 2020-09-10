const core = require("@actions/core");
const github = require("@actions/github");

async function main() {
  try {
    const { owner, repo, number } = github.context.issue;
    const github_token = core.getInput("github-token", {required: true});

    const client = new github.getOctokit(github_token);
    const { data: data } = await client.pulls.listCommits({
      owner: owner,
      repo: repo,
      pull_number: number,
    });
    const ref = data[0].parents[0].sha;
    console.log(`pr base ref is ${ref}`);
    core.setOutput("ref", ref);
  } catch (error) {
    core.setFailed(error.message);
  }
}

main();
