const core = require("@actions/core");
const github = require("@actions/github");

async function main() {
  try {
    const github_token = core.getInput("github-token", {required: true});
    const number = core.getInput("number", {required: true});
    const users = core.getInput("users", {required: false});
    const teams = core.getInput("teams", {required: false});

    if (!users && !teams) {
      core.warning("users and teams fields both empty");
      return;
    }

    const user_list = (users || "")
          .split(",")
          .map(s => s.trim())
          .filter(s => s.length > 0);
    const team_list = (teams || "")
          .split(",")
          .map(s => s.trim())
          .filter(s => s.length > 0);

    const client = new github.getOctokit(github_token);
    const repository = github.context.payload.repository;

    const result = await client.pulls.requestReviewers({
      owner: repository.owner.login,
      repo: repository.name,
      pull_number: number,
      reviewers: user_list,
      team_reviewers: team_list,
    });
  } catch (error) {
    core.setFailed(error.message);
  }
}

main();
