# Duniter git conventions

## TL;DR summary of this page, workflow instructions

The summary gives an overview of the rules described below. Reading it will help you to dive into the details.

- draft work must be prefixed by "WIP" (work in progress)
- the naming of final commits must comply with the template `type(scope): action subject`
- one should communicate with developers through dedicated spaces
- integrating a contribution can only be done via a merge request on our gitlab option and since the following critera are fullfilled
  - branch up to date with `master` branch (except hotfixes, see the hotfix section)
  - idiomatic code formatting, automated tests passed successfully
  - clean commit history, understandable and concise
  - contribution approved by a reviewer

## Naming commits

Every commit must comply with [conventional commit specification v1.0.0].

The commit name has to be meaningful in the context of commit history reread. It should not make reference to a specific MR or discussion.
Among other, commit history is used in changlogs and to track the project progress, that's why it has to be self-explanatory.
If you have a new need, please contact the main developers to add a type together.

## Update strategy

We only use **rebases**, *merges* are strictly fordbidden !

Every time the `master` branch is updated, you must rebase each of your working branch on it. For each of them:

1. Go on your branch
1. Rebase on master with `git rebase master`
1. If you see conflicts, fix them by editing the sources. Once it is done, you must:
   1. commit the files that were in conflict
   1. continue the rebase with `git rebase --continue`
1. Keep doing until you don't have any more conflict after `git rebase --continue`.

To prevent accidental merge commits, we recommend to force the `--ff-only` option on the merge command:

    git config --global merge.ff only

## When to push

Ideally, you should push when you are about to shut down your computer, so about once a day.

You must prefix your commit with `wip:` when it is a work in progress.

> But why push if I am not done ?

Pushing is no big deal and prevents you from loosing work in case of
any problem with your material.

## Before requesting proofreading of your merge request

After complying with the above criteria in your commits, you should check that your branch is up to date with the target branch (`master` in this example). As this branch is moving forward frequently, it is possible that new commits may have occurred while you were working on your branch (named YOUR_BRANCH, here). If this is the case or in case of doubt, to update your branch with respect to `master`, do the following:

```bash
git checkout master       # switch to master branch
git pull                  # updates the remote branch based on remote
git checkout YOU_BRANCH   # switch back to your branch
git rebase master         # rebase you work on master branch
```

In case of conflict during rebase that you can not solve, contact a lead developer telling them the hash of the commit on which YOUR_BRANCH is currently based so they can reproduce the rebase and see the conflicts. While waiting for their answer, you can cancel the rebase and work on YOUR_BRANCH without updating:

```
git rebase --abort
```

It is better to take your time before integrating a new contribution because the history of the master branch cannot be modified: it is a protected branch. Each commit on this branch remains there *ad vitam aeternam* that is why we make sure to keep a clear and understandable commit history.

## Discussion in a merge request

On Gitlab, a discussion is opened for each merge request. It will allow you to discuss the changes you have made. Feel free to tag someone by writing @pseudo so that they are notified of your request. Don't be impatient, the review of your contribution may take more or less time depending on its content!

The general discussion is used to comment on the merge request as a whole, for example to tag a developer for a proofreading request. When it comes to discussing a specific change in the code, you should go to the "Changes" tab of the merge request and comment under the code extract involved. This makes it easier to break down the resolution of problems raised by the merge request via the "comment resolution" feature. Each segment can be marked as resolved, but only the reviewer is allowed to do so!

## How to merge

When you finished developing, you must compile, run linter and run all tests:

    cargo fmt
    cargo clippy
    cargo tu
    cargo cucumber

Then commit everything.

In case you had a `wip:` prefix, you can remove it.

If you have a pile of commits, use the useful interactive rebase to clean up your branch history and create atomic ones:

    git rebase -i master

There you can rename the `wip:` commits, you can "fixup" commits that go together, you can rename and re-order commits,...

After an interactive rebase, your local git history differs from Gitlab's version, so you need a force push to make it to Gitlab:

    git push -f

Now is time to go to Gitlab and re-check your commits.

Wait for the Continuous Integration pipeline to finish (it lasts ±20min), and at last when it is done you can remove the "WIP" mention of your Merge Request and mention (with "@name") the lead developers to ask for a code review.

## Workflow

There are 3 types of permanent branches:

- The `master` branch is the default branch (the trunk), all contributions must be merged to this branch (except hotfixes).
- The `stable` branch, it always points to the most recent tag of the latest stable release. It is used as a reference for documentation, in particular.
- The hotfix branches, in `hotfix/x.y` format. A hotfix branch for an `x.y` release is only created when there is a patch to be released to production on that `x.y` release that cannot wait for the next release.

## Hotfix

If a blocking bug occurs in production and requires a hotfix, the latter must be the subject of 2 issues and 2 branches :

1. The original issue, must be processed on a `hotfix/issue-number-or-bug-description` branch, then merged to the `hotfix/x.y` branch, where `x.y` is the version in production at that time.
2. A carryover issue must be created, quoting the original issue and tracing the bug fix to the `master` branch. If for any reason the hotfix does not need to be carried over to the `master` branch, the carryover issue should explain why and then be closed.

[conventional commit specification v1.0.0]: https://www.conventionalcommits.org/en/v1.0.0/#specification
