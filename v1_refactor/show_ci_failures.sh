#!/bin/bash

count=0

# Use process substitution < <(...) so the loop runs in the current shell.
while IFS= read -r -d '' file; do
    
    # Check if the file contains the exact string.
    if grep -q '"all_succeeded": false' "$file"; then
        
        ((count++))
        
        echo "► PATH: $file"
        
        cat "$file"
        
        echo -e "\n---------------------------------------------------"
        
    fi

done < <(find /home/master/.local/share/argos/repo_tracking -maxdepth 1 -type f -name "*.json" -print0)

echo "Done! Total files with failures found: $count"
