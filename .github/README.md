
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
yor get --keys
```

