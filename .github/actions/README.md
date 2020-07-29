# github actions

These actions are custom for Libra Core workflows. Some of these actions are
hyperjump-powered, which means they allow specific, benign capabilities even
from forked repositories. Hyperjump actions consist of the action workflows
should use (ex. `comment`) and the backend hyperjump part that triggers on
`repository_dispatch` events (ex. `hyperjump-comment`).
