#!/bin/bash

declare -A links
while IFS= read -r -d '' n; do
	dirname="$(dirname "$n")"
	base="$(basename "$dirname")"
	links["$base"]="$dirname"

done < <(find ~/projects/ -iname ".git" -print0 -or -iname ".project" -not -path "*/.git/*" -print0)

printf '%s\n' "${links[@]}"
