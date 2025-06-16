# falconf

Falconf is a tool to manage your personal Linux machine(s). It is an imperative
alternative to [Nix](https://nixos.org/)
and [Ansible](https://www.ansible.com/), to make tweaking your computers
seamless without needing to edit files in a specialized syntax.

Falconf can track configuration files, and run arbitrary commands. These changes
are tracked and can be undone, and are synchronized with all your machines
through a Git repository. The ultimate goal is to be able to bring a clean Linux
install to your desired state with as few manual steps as possible, and to never
have to tweak something on multiple machines. More things can be automated than
you think!

## Installation

TODO

## Quickstart

TODO

TODO: how to sync, Topgrade

## Comparison to similar tools

If you want to compare Falconf to Ansible, there are two main differences:
1. Falconf tracks if a change is done, and does the change if it is not done yet,
regardless of any system state. Ansible does not track this, and instead determines
if the change is necessary based on the system state.
2. In Ansible, tasks are managed in a file that you need to edit. In Falconf, pieces
are managed (added, removed) on the command line, and only stored in a file internally.

- ✅: Yes
- ❌: No
- ➖: Possible but not built-in and/or against philosophy

TODO: Chezmoi
TODO: Stor
TODO: watch
| Characteristic                                        | Falconf | Nix | Ansible | Chezmoi | GNU Stow |
|-------------------------------------------------------|:-------:|:---:|:-------:|
| Reproducible                                          |    ✅    |  ✅  |    ✅    |
| Full functionality on all Linux distros               |    ✅    |  ❌  |    ✅    |
| (Smart) undo                                          |    ✅    |  ➖  |    ➖    |
| One-time                                              |    ✅    |  ❌  |    ❌    |
| Interact via CLI                                      |    ✅    |  ❌  |    ❌    |
| Built-in synchronization                              |    ✅    |  ❌  |    ❌    |
| Supports use on servers                               |    ❌    |  ✅  |    ✅    |
| Also a package manager                                |    ❌    |  ✅  |    ❌    |
| Extensive built-in support for specific programs etc. |    ❌    |  ✅  |    ❌    |
| Run arbitrary commands and edit arbitrary files       |    ✅    |  ➖  |    ✅    |
| Watch files (or dconf) and notify                     |    check |  no  |    no    |

Feel free to add more tools to the table!
