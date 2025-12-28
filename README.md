# GitHub garbage collector

Find and delete your old branches and forks, with associated PRs merged or closed.

For example, I have created a fork for
[Chatty](https://github.com/Brikster/Chatty) a few years ago and completely
forgot about it. I want to delete that fork but I don't want to manually check
if my PR got eventually merged. This is where this project comes in, it
automatically finds dead forks and proposes you to delete them.

> [!WARNING]
> This is still WIP, it doesn't ask you if you want to delete forks yet, but
> can find them (I hope correctly)

## Limitations

- Only checks forks, if there is a branch with merged PR on a main repo, this
  project won't find that branch.
- Cannot know if the default branch is ahead of the parent's fork branch, which
  means if you committed to e.g. `main` branch, this project cannot differ it
  from making changes only in other branches.
