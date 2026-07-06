# Shell completions

## Static scripts

```sh
twc-rs completions nushell > ~/.config/nushell/completions/twc-rs.nu
twc-rs completions zsh     > ~/.zfunc/_twc-rs
twc-rs completions bash    > /etc/bash_completion.d/twc-rs
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`, `nushell`.
The AUR package installs completions into the standard vendor directories.

## Dynamic completions

The dynamic engine also completes **live values** — `twc-rs apps logs <TAB>`
offers your actual apps by name and ID, fetched from the API (silently
skipped when offline).

![twc-rs dynamic completion demo](https://raw.githubusercontent.com/RAprogramm/twc-rs/main/docs/demo/completions.gif)

| Shell | Add to |
|---|---|
| bash | `echo 'source <(COMPLETE=bash twc-rs)' >> ~/.bashrc` |
| zsh | `echo 'source <(COMPLETE=zsh twc-rs)' >> ~/.zshrc` |
| fish | `echo 'COMPLETE=fish twc-rs \| source' >> ~/.config/fish/config.fish` |
| elvish | `echo 'eval (E:COMPLETE=elvish twc-rs \| slurp)' >> ~/.elvish/rc.elv` |
| powershell | `$env:COMPLETE = "powershell"; twc-rs \| Out-String \| Invoke-Expression; Remove-Item Env:\COMPLETE` in `$PROFILE` |

Nushell keeps the static script — the dynamic engine does not support it yet.
