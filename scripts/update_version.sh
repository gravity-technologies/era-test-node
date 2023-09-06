#!/bin/bash

set -e

# Extract version from tag
VERSION="$1"

# Validate version format
if [[ ! $VERSION =~ ^v?[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Invalid version format. Expected semantic versioning (e.g., v1.0.0 or 1.0.0)."
    exit 1
fi

# If tag has a 'v' prefix, remove it
if [[ $VERSION == v* ]]; then
    VERSION=${VERSION:1}
fi


# Update version in Cargo.toml
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

echo "Version updated to $VERSION in Cargo.toml."
