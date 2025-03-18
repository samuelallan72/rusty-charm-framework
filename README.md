# Rusty Charm Framework

An experimental, minimal framework for building charms for the Juju charm ecosystem.

## About

This is an experiment in using Rust to develop charms for Juju.
It was also a learning experience for how charms work at a lower level.

It's still a work in progress, rather experimental, and not stable at all.
See `./TODO.md` for ideas and things that still need to be implemented.


## Usage

For reference documentation:

```
cargo doc --open
```

See also the code under `./src/`, starting with `./src/lib.rs`.

Currently the repository also contains a proof of concept charm as a simple example of using the framework.
See the code in `./src/main.rs`.
To build and deploy the charm, use charmcraft v3 and juju:

```
charmcraft -v pack
juju deploy ./rusty_ubuntu@24.04-amd64.charm
```


## License

Rusty Charm Framework
Copyright (C) 2024-2025 Samuel Allan

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
