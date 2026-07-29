#!/bin/bash

err() {
	echo "Script failed: $1" >&2
	exit 1
}

if [ -z "${TRUNK_STAGING_DIR}" ]; then
	err "TRUNK_STAGING_DIR environment variable is not set"
fi

stagingDir="${TRUNK_STAGING_DIR/\\\\\\\?\\/}"

if ! [ -d "$stagingDir" ]; then
	err "Staging directory does not exist: $stagingDir"
fi

swPath="${stagingDir/%/\//}/sw.js"
backupPath="${stagingDir/%/\//}/.sw.js"

if ! [ -f "$swPath" ]; then
	err "Source file does not exist: $swPath"
fi

if ! cp "$swPath" "$backupPath"; then
	err "Failed to create temporary file"
fi

timestamp=$(date +%s) || err "Failed to get current timestamp"

tempPath=$(mktemp) || err "Failed to make temp file"

{ echo "const BUILD_TIME = $timestamp;"; cat "$swPath"; } > "$tempPath" || err "Failed to update service worker file"

if ! mv -f "$tempPath" "$swPath"; then
	if [ -f "$backupPath" ]; then
		mv -f "$backupPath" "$swPath" || true
	fi
	rm -f "$tempPath" || true
	err "Failed to update service worker file"
fi

rm -f "$backupPath" || true
