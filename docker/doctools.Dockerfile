ARG FNM_EXEC="/root/.fnm/fnm exec --using default"

FROM python:3.10-slim-bookworm AS node-setup

# Install fnm
RUN apt-get update \
  && apt-get install -y unzip curl \
  && curl -fsSL https://fnm.vercel.app/install \
  | bash -s -- --install-dir /root/.fnm --skip-shell

# Install Node
RUN ~/.fnm/fnm install 16.20 \
  && ~/.fnm/fnm default 16.20

FROM python:3.10-slim-bookworm AS pnpm-workspace

WORKDIR /repo

COPY . ./

# Remove everything that is not package.json
# https://stackoverflow.com/a/63142468/22226623
RUN find . -mindepth 1 -type f,l -not -name "package.json" -print | xargs rm -rf

FROM python:3.10-slim-bookworm AS build

ARG FNM_EXEC

WORKDIR /repo

# Install Python build backend
RUN pip install hatch

COPY --from=node-setup /root/.fnm/ /root/.fnm/

# Set up pnpm
RUN ${FNM_EXEC} npm install -g pnpm@8.6

# Copy only package.json which is needed for pnpm install
# This prevents source code changes from invalidating the cache
COPY --from=pnpm-workspace /repo/ ./

COPY pnpm-lock.yaml pnpm-workspace.yaml ./

# Install dependencies
# --ignore-scripts is needed to prevent pnpm from running build scripts
RUN ${FNM_EXEC} pnpm install --force --frozen-lockfile --ignore-scripts

# Copy the rest of the repo
COPY . ./

# Build scaffolding
RUN ${FNM_EXEC} pnpm exec nx run 'dumi-scaffolding:build'

# Build doctools
RUN ${FNM_EXEC} pnpm exec nx run 'doctools:build'

# Export artifacts
RUN ${FNM_EXEC} pnpm --filter dumi-scaffolding --prod deploy /build/scaffolding/
RUN mkdir -p /build/pypi-simple/secretflow-doctools/ \
  && cp ./pyprojects/secretflow-doctools/dist/* /build/pypi-simple/secretflow-doctools/

FROM python:3.10-slim-bookworm AS runtime

WORKDIR /workspace

# Prefetch dependencies
# https://github.com/mitsuhiko/rye/discussions/239#discussioncomment-6032595
COPY requirements.lock ./
RUN sed '/-e/d' requirements.lock > requirements.txt
RUN pip install -r requirements.txt

COPY --from=node-setup /root/.fnm/ /root/.fnm/
COPY --from=build /build/ ./

RUN find /workspace/pypi-simple/ -name '*.tar.gz' -exec pip install {} +

ENV STATIC_SITE_NODE_PREFIX="/root/.fnm/aliases/default"
ENV STATIC_SITE_WORKSPACE_ROOT="/workspace/scaffolding/"
ENV STATIC_SITE_INSTALL_DEPS="false"

ENV SIMPLE_PYPI_ROOT_DIRECTORY="/workspace/pypi-simple/"

# FIXME: mdserver still needs this to find node
ENV PATH="/root/.fnm/aliases/default/bin:${PATH}"

# devserver
EXPOSE 8000/tcp

# pypi
EXPOSE 8091/tcp

ENTRYPOINT secretflow-doctools $0 $@
