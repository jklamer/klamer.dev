#!/bin/sh
set -e

#change to script directory
cd "$(dirname "$0")"

# check if docker is running
if ! docker info > /dev/null 2>&1; then
  echo "Docker is not running"
  exit 1
fi

if [[ -z "${REPO}" ]]; then
  echo "REPO is not set"
  exit 1
fi

aws ecr get-login-password --region us-east-2 | docker login --username AWS --password-stdin $REPO

GIT_COMMIT=$(git rev-parse --short HEAD)
docker buildx build . -t $REPO:$GIT_COMMIT -t  $REPO:"latest" --platform linux/arm64 --push
