# grapevine_lib

[![Crates.io Version](https://img.shields.io/crates/v/grapevine_lib)](https://crates.io/crates/grapevine_lib)
[![docs.rs](https://img.shields.io/docsrs/grapevine_lib)](https://docs.rs/grapevine_lib)
[![License](https://img.shields.io/crates/l/grapevine_lib)](../LICENSE)

The [`grapevine`](https://github.com/TCA166/grapevine) backend. It features
a fully fledged out socket server, that can handle multiple clients separately
in threads, while providing fully encrypted communication between recipients.

## Features

The core feature of the library is the `GrapevineApp` struct, which provides a
facade over the toolkit. With this struct, you can create, manage, and interact
with channels. The app should self monitor its state and handle errors
gracefully. It's a fully fledged out threaded application, packaged neatly
in a single struct.
