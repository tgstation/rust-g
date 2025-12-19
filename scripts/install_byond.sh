#!/bin/bash
set -euo pipefail

# Detect OS
if [[ "$OSTYPE" == "win32" ]]; then
    IS_WINDOWS=1
    BYOND_SUFFIX=""
else
    IS_WINDOWS=0
    BYOND_SUFFIX="_linux"
fi

if [ -d "$HOME/BYOND/byond/bin" ] && grep -Fxq "${BYOND_MAJOR}.${BYOND_MINOR}" $HOME/BYOND/version.txt 2>/dev/null;
then
    echo "Using cached directory."
else
    echo "Setting up BYOND."
    rm -rf "$HOME/BYOND"
    mkdir -p "$HOME/BYOND"
    cd "$HOME/BYOND"

    if ! curl --connect-timeout 2 --max-time 8 "https://spacestation13.github.io/byond-builds/${BYOND_MAJOR}/${BYOND_MAJOR}.${BYOND_MINOR}_byond${BYOND_SUFFIX}.zip" -o byond.zip -A "GitHub Actions/1.0"; then
        echo "Mirror download failed, falling back to byond.com"
        if ! curl --connect-timeout 2 --max-time 8 "http://www.byond.com/download/build/${BYOND_MAJOR}/${BYOND_MAJOR}.${BYOND_MINOR}_byond${BYOND_SUFFIX}.zip" -o byond.zip -A "GitHub Actions/1.0"; then
            echo "BYOND download failed too :("
            exit 1
        fi
    fi

    unzip byond.zip
    rm byond.zip

    if [ "$IS_WINDOWS" -eq 0 ]; then
        cd byond
        make here
        cd ..
    fi

    echo "$BYOND_MAJOR.$BYOND_MINOR" > "$HOME/BYOND/version.txt"
    cd ~/
fi
