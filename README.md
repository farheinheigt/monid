# monid

Show system and user identity details from a small autonomous Rust CLI.

## Entrypoints

- User command: `bin/monid`
- Zsh completion: `bin/_monid.completion.zsh`
- Rust source: `src/main.rs`

## Usage

Run the command directly: `bin/monid`.
Generate completion script: `bin/monid --completion zsh`.

## Examples

`bin/monid`
`bin/monid --completion zsh`
`bin/monid root`

## Requirements

- Runtime wrapper: `zsh`
- Build tool: `cargo`

## Notes

- The user-facing entrypoint remains `bin/monid`.
- The Rust source lives under `src/`.
- The Cargo build output stays in the repository-local `target/` directory.
