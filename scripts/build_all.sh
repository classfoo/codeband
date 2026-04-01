#!/usr/bin/env bash
set -euo pipefail

npm install
npm run build:web
npm run build:server
npm run build:desktop
