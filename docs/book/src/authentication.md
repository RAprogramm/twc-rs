# Authentication

```sh
twc-rs auth flow                       # guided browser flow, stored in the OS keyring
# or
twc-rs config set-token --token <TOKEN>
```

The token is resolved in this order:

1. `--token` flag
2. `TWC_TOKEN` environment variable
3. OS keyring
4. config file (`~/.config/twc-rs/config.toml`)

## Profiles

Multiple accounts are supported via named profiles:

```sh
twc-rs config set-token --profile staging --token <TOKEN>
twc-rs --profile staging server list
```

The active profile can also be switched from inside the dashboard.
