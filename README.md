# grapevine

Small peer-to-peer end-to-end encrypted messanging app.

## Features

Grapevine allows users to connect to specific addresses and send messages
to each other. Optionally users can enable the internal server, allowing
other users to connect to themselves, with user permission of course.

### Encryption

Each connection is end-to-end encrypted using symmentric AES encryption.
The AES key exchange is encrypted using RSA encryption. Depending on
use input, those RSA keys can either be exchanged automatically,
or loaded externally, allowing for full control over the encryption.

### Verification

Each packet, following the RSA key exchange is signed using those RSA keys.
Thanks to this, provided the RSA key exchange is secure, the messages
are guaranteed to come from the trusted party.

## Building

```sh
cargo build --release
```

should do the trick. You will then find the built executable in the `target`
directory. If you wish to crosscompile for Windows:

```sh
rustup target add x86_64-pc-windows-gnu
cargo build --target x86_64-pc-windows-gnu
```

should work just fine.
