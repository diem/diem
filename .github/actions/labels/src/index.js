const core = require("@actions/core");
const github = require("@actions/github");
const got = require("got");
const process = require("process");

const hyperjump_url = "http://github.aws.hlw3truzy4ls.com:6080/hyperjump/jump";

async function main() {
  try {
    const { owner, repo, number } = github.context.issue;
    const hyperjump_url = core.getInput("hyperjump-url", {required: true});
    const add_labels = core.getInput("add", {required: false});
    const remove_labels = core.getInput("remove", {required: false});

    const add_list = (add_labels || "")
          .split(",")
          .map(s => s.trim())
          .filter(s => s.length > 0);
    const remove_list = (remove_labels || "")
          .split(",")
          .map(s => s.trim())
          .filter(s => s.length > 0);

    // trigger the hyperjump
    const body = {
      owner: owner,
      repo: repo,
      type: "labels",
      args: {
        number: number,
        add: add_list,
        remove: remove_list,
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
