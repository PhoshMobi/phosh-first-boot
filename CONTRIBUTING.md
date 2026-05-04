# Contributing to Phosh's first-boot

Thank you for considering contributing to this project. See below for
contributing guidelines.

Please make sure to check our [Code of Conduct][coc], for interactions
in this repository [GNOME's Code of Conduct][gnome-coc] also applies.

## Building

For build instructions, see the [README.md](./README.md)

## Merge requests

Before filing a pull request, run the tests:

```sh
meson test -C _build --print-errorlogs
```

Use descriptive commit messages, see

   <https://wiki.gnome.org/Git/CommitMessages>

and check

   <https://wiki.openstack.org/wiki/GitCommitMessages>

for good examples. The commits in a merge request should have "recipe"
style history rather than being a work log. See
[here](https://www.bitsnbites.eu/git-history-work-log-vs-recipe/) for
an explanation of the difference. The advantage is that the code stays
bisectable and individual bits can be cherry-picked or reverted.

## Coding

### Coding Style

For coding style see our [developer documentation][dev-docs].

[dev-docs]: http://dev.phosh.mobi/docs/development/coding-style/
[coc]: https://ev.phosh.mobi/resources/code-of-conduct/
[gnome-coc]: https://conduct.gnome.org/
