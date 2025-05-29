FROM ghcr.io/astral-sh/uv:0.7 AS uv

FROM docker.io/denoland/deno:bin-2.3.3 AS deno

FROM mcr.microsoft.com/devcontainers/typescript-node:20-bookworm

COPY --from=uv /uv /uvx /usr/local/bin/
COPY --from=deno /deno /usr/local/bin/

USER node

RUN npx -y playwright@1.52.0 install --with-deps

RUN pnpm config set store-dir $HOME/.pnpm-store
