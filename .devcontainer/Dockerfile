ARG VARIANT="bullseye"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

RUN export DEBIAN_FRONTEND=noninteractive && apt update && apt upgrade -y
RUN apt -y install --no-install-recommends mingw-w64

RUN rustup target add x86_64-pc-windows-gnu

RUN cargo install cargo-tarpaulin

RUN mkdir -p /workspaces/target
