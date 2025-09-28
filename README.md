# falconf

Falconf is a tool to manage your personal Linux machine(s). It is an imperative
alternative to [Nix](https://nixos.org/)
and [Ansible](https://www.ansible.com/), to make tweaking your computers
seamless without needing to edit files in a specialized syntax. Falconf puts
usability and speed of iteration before idempotency and correctness.

Falconf keeps track of changes done on top of your base Linux install, and syncs
then across your machines.

Falconf can track configuration files, and run arbitrary commands. These changes
are tracked and can be undone, and are synchronized with all your machines
through a Git repository. The ultimate goal is to be able to bring a clean Linux
install to your desired state with as few manual steps as possible, and to never
have to tweak something manually on multiple machines. More things can be automated than
you think!

### Falconf is for you if...

* You have a script to set up a new Linux machine, that you update (semi-)regularly
* You have a list of manual actions to take to set up a new Linux machine
* You use Ansible to manage your machines, but find it cumbersome to add/remove tasks
* You use a home-made script / plain git server for synchronizing dotfiles
* You regularly forget / have to remind yourself to install a program / tweak something on your other machines as well
* You're afraid to re-install because you'll lose your configuration

### Falconf is **not** for you if...

* You're happy with NixOS
* You use an almost clean Linux install

## Installation

```bash
# Using cargo-binstall:
cargo binstall falconf
# Using cargo:
cargo install falconf
```

You can also manually download the binary from the releases page, but that can't
update itself (yet).

## Usage

### Quickstart

1. Create a remote Git repository, for example on GitHub.
   A good name is `my-falconf`. This repository can be private.
2. Run `falconf init --new <remote_url>`, where `<remote_url>`
   is the Git url of the remote.
3. On other machines, run `falconf init <remote_url>`, then `falconf sync`.

Any changes are automatically pulled from and pushed to the Git repository,
but not automatically executed. For that, use `falconf sync`.
[Topgrade](https://github.com/topgrade-rs/topgrade) can also do this for you.

### Example usage

Let's say you just discovered [duf](https://github.com/muesli/duf), and want
to use it instead of `df`. You add an alias `alias df=duf` to `~/.bash_aliases`,
which is tracked with `falconf add -f ~/.bash_aliases`. You then run
`falconf push`, which pushes the changes you made in `~/.bash_aliases` (and shows
you a diff), and `falconf add -n apt install duf`, which installs `duf`. When you then
run `falconf sync` on your other machine, it adds the alias to `~/.bash_aliases`,
and installs `duf`.

If you decide you actually want to use a different tool, like
[dysk](https://github.com/canop/dysk), run `falconf list` to find the `apt install duf`
piece, and run `falconf undo -n <piece id>`, where `<piece id>` is the 8-digit hexadecimal
ID noted in brackets in the `falconf list` output. This automatically runs `apt remove --autoremove duf`
for you, and on your other machines, and marks the piece for deletion when every machine has.
This way you don't clutter your pieces with install and remove commands. You can then edit
`~/.bash_aliases` and run `falconf push` again, and run `falconf add -n cargo binstall dysk`.
The next sync on the other machine will then update `~/.bash_aliases`, uninstall `duf`,
and install `dysk`.

### Tips

* Running `falconf add` without `--not-done-here` (`-n`) will assume you've already ran the command
  here. You can for example run any command, and then run `falconf add !!`. Your shell will expand `!!` to the
  previous command you ran.

## Comparison to similar tools

The most similar tool to falconf is Ansible, but there are two main differences:

1. Falconf tracks if a change is done, and does the change if it is not done yet,
   regardless of any system state. Ansible does not track this, and instead determines
   if the change is necessary based on the system state. This means that falconf is less
   reliable, but easier to use.
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
| dconf support (specific paths)*                  |    ⏳    |  ✅  |    ✅    |    ❌    |    ❌     |
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
