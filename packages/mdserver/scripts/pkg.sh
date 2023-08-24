#!/usr/bin/env bash

pkg -t node16-macos-x64,node16-linux-x64,node16-windows-x64 \
  --out-path dist/bin \
  --compress GZip \
  dist/mdserver.cjs
