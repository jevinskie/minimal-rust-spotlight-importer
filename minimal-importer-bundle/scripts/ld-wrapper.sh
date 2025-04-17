#!/usr/bin/env bash

set -xeou pipefail

if [[ "${CARGO_CRATE_NAME}" == "minimal_importer_bundle" ]]; then
    # Collect arguments, swap -dynamiclib with -bundle
    args=()
    for arg in "$@"; do
        if [[ "$arg" != "-dynamiclib" ]]; then
            args+=("$arg")
        else
            args+=("-bundle")
        fi
    done
else
    args=($@)
fi

# Call the real linker with modified arguments
exec cc "${args[@]}"
