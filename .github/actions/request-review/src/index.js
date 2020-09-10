const core = require("@actions/core");
const github = require("@actions/github");
const got = require("got");
const process = require("process");

const hyperjump_url = "http://github.aws.hlw3truzy4ls.com:6080/hyperjump/jump";

async function main() {
  try {
    const { owner, repo, number } = github.context.issue;
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

    // trigger the hyperjump
    const body = {
      owner: owner,
      repo: repo,
      type: "request-review",
      args: {
        number: number,
        users: user_list,
        teams: team_list,
      },
    };
    await got.post(hyperjump_url, {
      retry: 0,
      json: body,
    });
  } catch (error) {
    core.setFailed(error.message);
  }
}

main();
