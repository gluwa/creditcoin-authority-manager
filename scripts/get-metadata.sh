#!/usr/bin/env bash

set -e

if ! command -v subxt > /dev/null
then
    cargo install subxt-cli
fi

if ! curl -H "Content-Type: application/json" --data '{ "jsonrpc":"2.0", "method":"system_health", "params":[],"id":1 }' localhost:9933 2> /dev/null
then
    echo "unreachable node on deafult port, cant fetch node metadata."
    exit 1
fi

subxt metadata > creditcoin-metadata.scale
