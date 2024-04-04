# Miner ⛏️

[![test status](https://github.com/matteopolak/miner/actions/workflows/ci.yml/badge.svg)](.github/workflows/ci.yml)
[![release status](https://github.com/matteopolak/miner/actions/workflows/release.yml/badge.svg)](.github/workflows/release.yml)
[![license](https://img.shields.io/github/license/matteopolak/miner.svg)](LICENSE)

A GPU and CPU solo miner for Bitcoin.

```powershell
Usage: miner [OPTIONS] --username <USERNAME> --password <PASSWORD> --address <ADDRESS>

Options:
  -u, --username <USERNAME>  RPC username [env: RPC_USERNAME=]
  -p, --password <PASSWORD>  RPC password [env: RPC_PASSWORD=]
  -a, --address <ADDRESS>    RPC address url [env: RPC_ADDRESS=]
  -g, --gpu                  Use the GPU for mining
  -h, --help                 Print help
  -V, --version              Print version
```

## Features

- Solo CPU and GPU mining
- Modern Bitcoin Core RPC
- Automatic wallet address generation
- Automatic difficulty adjustment
