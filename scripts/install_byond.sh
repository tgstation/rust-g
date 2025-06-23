#!/bin/bash
set -euo pipefail

if [ -d "$HOME/BYOND/byond/bin" ] && grep -Fxq "${BYOND_MAJOR}.${BYOND_MINOR}" $HOME/BYOND/version.txt;
then
    echo "Using cached directory."
else
    echo "Setting up BYOND."
    rm -rf "$HOME/BYOND"
    mkdir -p "$HOME/BYOND"
    cd "$HOME/BYOND"
    if ! curl --connect-timeout 3 --max-time 10 "http://www.byond.com/download/build/${BYOND_MAJOR}/${BYOND_MAJOR}.${BYOND_MINOR}_byond_linux.zip" -o byond.zip -A "GitHub Actions/1.0"; then
        echo "BYOND download failed or timed out, using mirror"
        if ! curl --connect-timeout 3 --max-time 10 "https://spacestation13.github.io/byond-builds/${BYOND_MAJOR}/${BYOND_MAJOR}.${BYOND_MINOR}_byond_linux.zip" -o byond.zip -A "GitHub Actions/1.0"; then
            echo "Mirror download failed too :("
            exit 1
        fi
    fi
    unzip byond.zip
    rm byond.zip
    cd byond
    make here
    echo "$BYOND_MAJOR.$BYOND_MINOR" > "$HOME/BYOND/version.txt"
    cd ~/
fi
