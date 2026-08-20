#!/usr/bin/env bash

prog_name=$(basename "$0")

items=("$prog_name" "$@")

echo "Number of args passed: ${#items[@]}"

i=0
for item in "${items[@]}"; do
    echo "#$i: $item"
    ((i++))
done
