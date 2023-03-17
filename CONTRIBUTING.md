# Contributing

Before contributing, please make sure that your development environment is properly configured by following this tutorial:

[Setting up your development environment]

Sign-ups on our gitlab are disabled. If you would like to contribute, please ask for an account on [the technical forum].

When contributing to this repository, please first discuss the change you wish to make via issue or
via [the technical forum] before making a change.

Please note we have a specific workflow, please follow it in all your interactions with the project.

## Developer documentation

Please read [Developer documentation] before contributing.

## Workflow

- If there is an unassigned issue about the thing you want to contribute to, assign the issue to yourself.
- Create a branch based on `master` and prefixed with your nickname. Give your branch a short name indicating the subject.
- Create an MR from your branch to `master`. Prefix the title with `Draft: ` until it's ready to be merged.
- If the MR is related to an issue, mention the issue in the description using the `#42` syntax.
- Never push to a branch of another contributor! If the contributor makes a `git rebase` your commit will be lost!
- Before you push your commit:
  - Apply formatters (rustfmt and prettier) and linter (clippy)
  - Document your code
  - Apply the [project's git conventions]

## Merge Process

1. Ensure you rebased your branch on the latest `master` commit to avoid any merge conflicts.
1. Ensure that you respect the [commit naming conventions].
1. Ensure that all automated tests pass with the `cargo test` command.
1. Ensure that the code is well formatted `cargo fmt` and complies with the good practices `cargo clippy`. If you have been working on tests, check everything with `cargo clippy --all --tests`.
1. Update the documentation with details of changes to the interface, this includes new environment variables, exposed ports, useful file locations and container parameters.
1. Push your branch on the gitlab and create a merge request. Briefly explain the purpose of your contribution in the description of the merge request.
1. Mark the MR as ready (or remove the `Draft: ` prefix) only when you think it can be reviewed or merged.
1. Assign a Duniter reviewer so they will review your contribution. If you still have no news after several weeks, ask explicitly for a review, or tag another reviewer or/and talk about your contribution on [the technical forum].

## List of Duniter's reviewers

- @HugoTrentesaux
- @tuxmain

[commit naming conventions]: ./docs/dev/git-conventions.md#naming-commits
[Developer documentation]: ./docs/dev/
[project's git conventions]: ./docs/dev/git-conventions.md
[Setting up your development environment]: ./docs/dev/setup.md
[the technical forum]: https://forum.duniter.org
