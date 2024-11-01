# pdpw

A very simple password store, that basically stores an encrypted textfile
containing whatever you did put there.

WIP: This is still very much in progress. So don't use it for now :)

## How to to install 

```shell
cargo install --path .
```

Clearly the above only works if the Rust compiler is installed as described
[here](https://www.rust-lang.org/tools/install)

## First steps

If you start pdpw without providing a *.pdpw file it will create a
`default.pdpw` in your HOME directory. Make sure to remember the password you
initially use.


## Basic shortcuts

- `strg + s` encrypt and save changes to the *.pdpw file that you have opened.
  Typically `default.pdpw`
- `strg + f` open the search dialog

## Configure Gnome Desktop integration

Add following line to `/etc/mime.types`

```text
application/pdpw                                pdpw
```

Now create a `pdpw.desktop` file in either `~/.local/share/applications` or
`/usr/share/applications` in case you want to configure `pdpw` for all users.

The desktop file should contain similar contents

```text
[Desktop Entry]
Type=Application
Name=PD Password
Comment=Encrypted File as Password Manager
Icon=/usr/share/icons/breeze/mimetypes/16/application-pgp-keys.svg
Exec=pdpw %F
Terminal=true
Categories=Utility;TextEditor
```
