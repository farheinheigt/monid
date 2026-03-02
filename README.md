# monid

## Overview
Show system and user identity details.

## Location
- Repository: `/Users/farheinheigt/Projets/system/monid`
- User entrypoint: `/Users/farheinheigt/Projets/system/monid/bin/monid`
- Completion file: `/Users/farheinheigt/Projets/system/monid/bin/_monid.completion.zsh`

## Usage
Run the command directly: `monid`.
Generate completion script: `monid --completion zsh`.

## Examples
`monid`
`monid --completion zsh`
`monid root`

## Requirements
- Runtime wrapper: `zsh`
- Build tool: `cargo`

## Notes
- The user-facing entrypoint remains `bin/monid`.
- The Rust source lives under `src/`.
- The Cargo build output stays in the repository-local `target/` directory.
