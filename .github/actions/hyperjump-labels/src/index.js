const core = require("@actions/core");
const github = require("@actions/github");

async function main() {
  try {
    const github_token = core.getInput("github-token", {required: true});
    const number = core.getInput("number", {required: true});
    const add_labels = core.getInput("add", {required: true});
    const remove_labels = core.getInput("remove", {required: true});

    if (!add_labels && !remove_labels) {
      core.warning("add and remove fields both empty");
      return;
    }

    const add_list = (add_labels || "")
          .split(",")
          .map(s => s.trim())
          .filter(s => s.length > 0);
    const remove_list = (remove_labels || "")
          .split(",")
          .map(s => s.trim())
          .filter(s => s.length > 0);

    const client = new github.getOctokit(github_token);
    const repository = github.context.payload.repository;

    if (add_list.length > 0) {
      await client.issues.addLabels({
        owner: repository.owner.login,
        repo: repository.name,
        issue_number: number,
        labels: add_list,
      });
    }

    if (remove_list.length > 0) {
      await Promise.all(
        remove_list.map(label => {
          client.issues.removeLabel({
            owner: repository.owner.login,
            repo: repository.name,
            issue_number: number,
            name: label,
          });
        })
      );
    }
  } catch (error) {
    core.setFailed(error.message);
  }
}

main();
