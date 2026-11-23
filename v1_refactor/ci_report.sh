#!/bin/bash

count=0

# Use process substitution < <(...) so the loop runs in the current shell.
while IFS= read -r -d '' file; do
    ((count++))
    
    echo "► PATH: $file"
    
    cat "$file"
    
    echo -e "\n---------------------------------------------------"
done < <(find /home/master/.local/share/argos/repo_tracking -maxdepth 1 -type f -name "*.json" -print0)

echo "Done! Total JSON files found: $count"
