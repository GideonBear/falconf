# falconf

Falconf is a tool to manage your personal Linux machine(s). It is an imperative
alternative to [Nix](https://nixos.org/)
and [Ansible](https://www.ansible.com/), to make tweaking your computers
seamless without needing to edit files in a specialized syntax. Falconf puts
usability and speed of iteration before idempotency and correctness.

Falconf can track configuration files, and run arbitrary commands. These changes
are tracked and can be undone, and are synchronized with all your machines
through a Git repository. The ultimate goal is to be able to bring a clean Linux
install to your desired state with as few manual steps as possible, and to never
have to tweak something on multiple machines. More things can be automated than
you think!

## Installation

```bash
# Using cargo-binstall:
cargo binstall falconf
# Using cargo:
cargo install falconf
```

You can also manually download the binary from the releases page.

## Quickstart

1. Create a remote Git repository, for example on GitHub.
   A good name is `my-falconf`. This repository can be private.
2. Run `falconf init --new <remote_url>`, where `<remote_url>`
   is the Git url of the remote.
3. On other machines, run `falconf init <remote_url>`, then `falconf sync`.

Any changes are automatically pulled from and pushed to the Git repository,
but not automatically executed. For that, use `falconf sync`.
[Topgrade](https://github.com/topgrade-rs/topgrade) can also do this for you.

## Comparison to similar tools

The most similar tool to falconf is Ansible, but there are two main differences:

1. Falconf tracks if a change is done, and does the change if it is not done yet,
   regardless of any system state. Ansible does not track this, and instead determines
   if the change is necessary based on the system state. This means that falconf is less
   reliable, but also easier to use.
2. In Ansible, tasks are managed in a file that you need to edit. In Falconf, pieces
   are managed (added, removed) on the command line, and only stored in a file internally.
   You can still edit this file if you want to, but do this at your own risk.

- ✅: Yes
- ❌: No
- ➖: Possible but not built-in and/or against philosophy
- ⏳: Planned

[//]: # (- ❓: Unknown, contribution welcome)

[//]: # (TODO: Everything that's ⏳ needs to be implemented)

| Characteristic                                   | Falconf | Nix | Ansible | Chezmoi | GNU Stow |
|--------------------------------------------------|:-------:|:---:|:-------:|:-------:|:--------:|
| Reproducible                                     |    ✅    |  ✅  |    ✅    |    ✅    |    ✅     |
| Idempotent design                                |    ❌    |  ✅  |    ✅    |    ✅    |    ✅     |
| Run arbitrary idempotent commands                |    ✅    |  ✅  |    ✅    |    ✅    |    ❌     |
| Run arbitrary non-idempotent commands            |    ✅    |  ❌  |    ❌    |    ❌    |    ❌     |
| Works with your previous configuration           |    ✅    |  ❌  |    ✅    |    ✅    |    ✅     |
| Full functionality on all Linux distros          |    ✅    |  ❌  |    ✅    |    ✅    |    ✅     |
| Interact (run commands, add files) via CLI       |    ✅    |  ❌  |    ❌    |    ❌    |    ❌     |
| Edit arbitrary files                             |    ✅    |  ➖  |    ✅    |    ✅    |    ✅     |
| Undo commands (with commands)                    |    ✅    |  ➖  |    ➖    |    ➖    |    ❌     |
| Undo changes (automatically, without commands)   |    ✅    |  ✅  |    ➖    |    ❌    |    ❌     |
| Runs without Git installation                    |    ✅    |  ✅  |    ✅    |    ✅    |    ✅     |
| Built-in synchronization                         |    ✅    |  ❌  |    ❌    |    ✅    |    ❌     |
| dconf support (specific paths)*                  |    ⏳    |  ❌  |    ❌    |    ❌    |    ❌     |
| Temporary one-time pieces                        |    ⏳    |  ❌  |    ❌    |    ❌    |    ❌     |
| Watch configuration (files, dconf)               |    ⏳    |  ❌  |    ❌    |    ❌    |    ❌     |
| Secret management                                |    ⏳    |  ✅  |    ✅    |    ✅    |    ❌     |
| Windows support                                  |    ⏳    |  ❌  |    ✅    |    ✅    |    ❌     |
| Topgrade integration                             |    ⏳    |  ✅  |    ❌    |    ✅    |    ❌     |
| Self-updating                                    |    ⏳    |  ✅  |    ❌    |    ✅    |    ❌     |
| Machine-to-machine differences (templates)       |    ⏳    |  ✅  |    ✅    |    ✅    |    ❌     |
| Supports use on servers                          |    ❌    |  ✅  |    ✅    |    ❌    |    ❌     |
| Made specifically for managing personal machines |    ✅    |  ✅  |    ❌    |    ✅    |    ✅     |
| Also a package manager                           |    ❌    |  ✅  |    ❌    |    ❌    |    ❌     |
| Extensive built-in support for specific programs |    ❌    |  ✅  |    ❌    |    ❌    |    ❌     |

\* Chezmoi supports syncing the entire dconf file via onchange scripts, but this is not always desired, because
things like window positions are stored in there as well. Falconf supports syncing specific dconf paths natively.
