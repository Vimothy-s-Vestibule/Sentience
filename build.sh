#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
touch backend/.env
touch frontend/.env
cmd="docker buildx bake --push $@"
echo ">>> $cmd"
$cmd
