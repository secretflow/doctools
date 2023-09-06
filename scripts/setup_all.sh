#!/usr/bin/env bash

# Bootstrap this monorepo for development

# Setup Node environment

pnpm install --frozen-lockfile

# Setup Python environment

$(dirname $0)/setup_python.sh

# Run setup tasks

pnpm exec nx run-many -t setup
