
# :old_key: Yor

Personal key-value storage with encryption.

Yor is yet another key-value storage with encryption designed for folks who store sensitive information.

```sh
# Set the key and value & prompt for the password
yor set hello world

# Set the key and value without prompting the password
yor set secret key --no-password

# Get the value of the given key
yor get hello 

# get the value of all keys
yor ls

# v0.0.2 now support different files. --type format: <supported_type>/<extension>
yor set image ~/Downloads/image.png --type image/png

# Binaries
yor set yorbin ~/.local/bin/yor --type file/bin

# To extract the file / images:
yor get image
# /home/zenn/.yor/files/image.png

yor get yorbin
# /home/zenn/.yor/files/yorbin
```

## Features
Here are some notable features on why Yor is a better option
- 100% open-sourced
- Use symmetric encryption to avoid several data breaching
- Optimized for privacy & security
