#!/bin/bash

# This script deploys the application to a remote server using SSH and rsync.
# It reads the deplyment server from .env file and uses the SSH key for authentication.

# Load environment variables from .env file
if [ -f .env ]; then
  source .env
else
  echo ".env file not found!"
  exit 1
fi

# Check if DEPLOY_SERVER is set
if [ -z "$DEPLOY_SERVER" ]; then
  echo "DEPLOY_SERVER is not set in .env file!"
  exit 1
fi

# Check if DEPLOY_PATH is set
if [ -z "$DEPLOY_PATH" ]; then
  echo "DEPLOY_PATH is not set in .env file!"
  exit 1
fi

# TODO
echo "TODO: Implement deployment logic here"