Rs-Zip
-------
Rs-Zip is a pure Rust-based CLI tool that implements custom data compression and symmetric encryption — all built using only the Rust standard library, with no external dependencies.

This project is designed for educational and experimental purposes, illustrating how classic compression and encryption algorithms can be implemented from scratch. It includes:

    Building a Huffman tree and encoding data with it.

    Implementing LZ77 compression to reduce redundancy.

    Using a Feistel network to build a symmetric block cipher.

Important Notice
-----------------
Rs-Zip’s encryption is NOT (yet) production-grade or cryptographically secure.
I plan to add my own secure AES (or other) encryption implementations using only Rust's standard library.

This tool is meant for learning, experimentation, and personal use only.

Building and Running
--------------------
Make sure you have Rust installed: https://rustup.rs/

Clone the repository and run:

    cargo run

Then follow the on-screen options to compress or encrypt files.
