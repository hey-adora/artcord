#!/bin/bash

# Detect the OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
elif [[ "$OSTYPE" == "cygwin" || "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    OS="windows"
else
    echo "Unknown OS"
    exit 1
fi

# Detect the architecture
if [[ "$(uname -m)" == "x86_64" ]]; then
    ARCH="x64"
elif [[ "$(uname -m)" == "arm64" ]]; then
    ARCH="arm64"
elif [[ "$(uname -m)" == "armv7l" ]]; then
    ARCH="armv7"
else
    echo "Unknown architecture"
    exit 1
fi

URL="https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-$OS-$ARCH"

# If the OS is Windows, print a message and exit
if [[ "$OS" == "windows" ]]; then
    echo "Get Linux."
    exit 1
fi

# Set the filename
FILENAME="style/tailwindcss"

# Download the file
curl -Lo $FILENAME $URL

# Make the file executable
chmod +x $FILENAME