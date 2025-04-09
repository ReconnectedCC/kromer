# Kromer

Centralized economy server for Minecraft servers.

# Development

Kromer is currently written in Rust, uses SurrealDB for a database, and is deployed using Docker.

## Rust and Cargo setup

Ensure that your Rust toolchain is [installed](https://www.rust-lang.org/tools/install) and [properly configured](https://rust-lang.github.io/rustup/concepts/toolchains.html).

Running `cargo build` or `cargo run` will automatically pull the dependencies, and compile a binary executable.

To simplify database schema maintenance, the cargo package `surrealdb-migrations` is used. To work with applying migrations manually, run `cargo install surrealdb-migrations`. You'll need to make sure that your Cargo binaries directory is correctly setup with your PATH variables (typically, this looks like adding `~/.cargo/bin` to your PATH).

## SurrealDB

The easiest way to get started with SurrealDB is to use Docker Compose, you can find a starter `docker-compose.yml` [in this repository](https://github.com/surrealdb/docker.surrealdb.com/blob/main/docker-compose.yml).

After you have Surreal running, you should configure your `.env` file to match your docker configuration. Also, create the file `.surrealdb` in the root directory based off the example. The executable will let you know if there are any configuration errors.

The database can be inspected by visting https://surrealist.app and connecting, then selecting the appropriately named namespace and database per your compose/env variables.

## SurrealQL Extension

It is recommended to use the extension [`SurrealQL`](https://marketplace.visualstudio.com/items?itemName=surrealdb.surrealql) inside of VSCode when editing SurrealQL files.
