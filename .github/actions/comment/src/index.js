const core = require("@actions/core");
const github = require("@actions/github");
const got = require("got");
const process = require("process");

const hyperjump_url = "http://github.aws.hlw3truzy4ls.com:6080/hyperjump/jump";

async function main() {
  try {
    const { owner, repo, number } = github.context.issue;
    const comment = core.getInput("comment", {required: true});
    const tag = core.getInput("tag", {required: false});
    const delete_older = core.getInput("delete-older", {required: false}) || false;

    // trigger the hyperjump
    const body = {
      owner: owner,
      repo: repo,
      type: "comment",
      args: {
        number: number,
        comment: comment,
        tag: tag,
        delete_older: delete_older,
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
