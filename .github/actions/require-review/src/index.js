const core = require("@actions/core");
const github = require("@actions/github");

async function main() {
  try {
    const github_token = core.getInput("github-token", {required: true});
    const users = core.getInput("users", {required: true});

    const user_list = (users || "")
          .split(",")
          .map(s => s.trim())
          .filter(s => s.length > 0);

    const repo = github.context.payload.repository;
    const pull = github.context.payload.pull_request;
    const client = new github.getOctokit(github_token);

    const { data } = await client.pulls.listReviews({
      owner: repo.owner.login,
      repo: repo.name,
      pull_number: pull.number,
    });

    const approvals = data
          .filter(r => r.state == "APPROVED")
          .map(r => r.user.login);

    const found = user_list.filter(u => approvals.find(name => name == u));

    if (found.length > 0) {
      console.log(`required approvals found from: ${found.join(", ")}`);
    } else {
      core.setFailed(`required reviews are missing from: ${user_list.join(", ")}`);
    }
  } catch (error) {
    core.setFailed(error.message);
  }
}

main();
