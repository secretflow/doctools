#!/usr/bin/env bash

eval $(fnm env)
source $HOME/.rye/env

REPO_ROOT=$(git rev-parse --show-toplevel)

if [ $? != 0 ]; then
  echo "Could not find repository root"
  exit 1
fi

cd $REPO_ROOT

# node_modules and .venv are mounted as a volume, fix permissions
sudo chown -R "$(id -un):$(id -gn)" node_modules .venv

# set pnpm to use node_modules/.pnpm-store as store-dir to support hardlinks
pnpm config set store-dir $PWD/node_modules/.pnpm-store

# preemptively create venv
PYTHON_TOOLCHAIN=$(rye show | sed -Enr 's/venv python: (.+)/\1/p' | tr -d "[:blank:]\t\n\r")

if [ $? == 0 ]; then
  $HOME/.rye/py/$PYTHON_TOOLCHAIN/install/bin/python3 -m venv .venv
  printf '{"python": "%s"}' "$PYTHON_TOOLCHAIN" >".venv/rye-venv.json"
else
  echo "Could not determine Python toolchain, skipping venv creation."
fi
