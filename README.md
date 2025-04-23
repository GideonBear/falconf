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

- ✅: Yes
- ❌: No
- ➖: Possible but not built-in and/or against philosophy

| Characteristic                                        | Falconf | Nix | Ansible |
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
