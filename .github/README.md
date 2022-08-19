
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
yor list-keys

# v0.0.2 now support different files
yor set image ~/Downloads/image.png --type image/png

# Binaries
yor set yorbin ~/.local/bin/yor --type file/bin

# To extract the file / images:
yor get image
# /home/zenn/.yor/files/image.png

yor get yorbin
# /home/zenn/.yor/files/yorbin
```

## Whats New~! :label: **v0.0.2**
> Status: not yet released
- Files are now supported such as **image**, **video**, and **files**
- Added new commands such as **clear**, **list-files** & more
- Improve serializing method. use **YorData** struct instead


## Features
Here are some notable features on why Yor is a better option
- No tracking involved
- 100% open-sourced
- Use symmetric encryption to avoid several data breaching
- Blazingly fast (it's written in Rust tho)
- Optimized for privacy & security
