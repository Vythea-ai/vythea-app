# Getting started

## Requirements

- **Windows** 11, x64
- **macOS** 13 or later, Apple silicon
- **Linux** x64, glibc 2.31 or later

## Install

Download the latest build from [Releases](https://github.com/Vythea-ai/vythea-app/releases/latest):

- **Windows**: `vythea-windows-x64.zip`. Unzip it and run `vythea.exe setup`.
- **macOS**: `vythea-macos-arm64.dmg`. Open it and drag Vythea into Applications.
- **Linux**: `vythea-linux-x64.tar.gz`. Unpack it and run `./vythea setup`.

Nothing needs administrator rights. Each release also carries `install.ps1` and `install.sh`, which download the latest release and hand over to `vythea setup`.

## Setup

`vythea setup` asks:

1. What to call you
2. Whether to sign in to your Vythea account (optional if you bring your own keys)
3. Which gateway and model to use
4. Whether to install the window, the terminal, or both

Run it again any time to change the answers. It also runs on first start when nothing is configured.

## First message

```sh
vythea
```

Type a message. Vythea asks before it runs a command or changes a file. `/help` lists the commands, and `/model` switches models.

## Uninstall

```sh
vythea --uninstall
```

On Windows, Apps & features also works. Uninstalling keeps your data folder (see [Data and privacy](data-and-privacy.md)).
