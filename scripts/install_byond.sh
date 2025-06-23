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
    if ! curl --connect-timeout 3 "http://www.byond.com/download/build/${BYOND_MAJOR}/${BYOND_MAJOR}.${BYOND_MINOR}_byond_linux.zip" -o byond.zip -A "GitHub Actions/1.0"; then
        echo "BYOND download failed, using mirror"
        curl "https://spacestation13.github.io/byond-builds/${BYOND_MAJOR}/${BYOND_MAJOR}.${BYOND_MINOR}_byond_linux.zip" -o byond.zip -A "GitHub Actions/1.0"
    fi
    unzip byond.zip
    rm byond.zip
    cd byond
    make here
    echo "$BYOND_MAJOR.$BYOND_MINOR" > "$HOME/BYOND/version.txt"
    cd ~/
fi
