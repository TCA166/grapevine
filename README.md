# grapevine

Small peer-to-peer end-to-end encrypted messanging app.

## Encryption

Each connection is end-to-end encrypted using symmentric AES encryption.
The AES key exchange is encrypted using RSA encryption. Depending on
use input, those RSA keys can either be exchanged automatically,
or loaded externally, allowing for full control over the encryption.

## Verification

Each packet, following the RSA key exchange is signed using those RSA keys.
Thanks to this, provided the RSA key exchange is secure, the messages
are guaranteed to come from the trusted party.
