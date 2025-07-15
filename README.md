Rs-Zip

Rs-Zip is a pure Rust-based CLI tool that implements custom data compression and symmetric encryption — all built using only the Rust standard library with no external dependencies.

    Huffman Coding: Efficient entropy encoding using a Huffman tree.

    LZ77 Compression: Sliding-window lossless data compression algorithm.

    Custom Feistel Cipher: Simple block cipher implementation based on a Feistel network for encrypting and decrypting files.

    Interactive CLI: Easy-to-use command-line interface for compressing, decompressing, encrypting, and decrypting files.

Why Rs-Zip?

This project is designed for educational and experimental purposes, illustrating how classic compression and encryption algorithms can be implemented from scratch. It demonstrates:

    How to build a Huffman tree and encode data with it.

    How to apply LZ77 compression to reduce redundancy.

    How a Feistel network can be used to build a symmetric block cipher.

    How to integrate these techniques into a CLI tool.

Important Notice

Rs-Zip’s encryption is NOT production-grade or cryptographically secure.
It is meant for learning and experimentation only. For real-world security, use well-vetted cryptographic libraries such as aes crates or system-provided crypto APIs.
Usage

Run the program and follow the menu options:

=== Rs-Zip CLI Tool ===
1) Compress file
2) Decompress file
3) Encrypt file
4) Decrypt file
5) Exit
Choose option:

You will be prompted for input and output file paths, and a key for encryption/decryption.
How it Works

    Compression:

        Input data is first compressed with LZ77 to reduce repeated sequences.

        The LZ77 output is serialized and then Huffman compressed for further size reduction.

        The Huffman tree is serialized and stored along with the compressed data.

    Decompression:

        Reads the serialized Huffman tree and decompresses Huffman data.

        Deserializes LZ77 tokens and decompresses to restore the original file.

    Encryption:

        Uses a custom Feistel network block cipher with a key derived from your input key string.

        Encrypts the file data in 64-bit blocks.

    Decryption:

        Applies the Feistel cipher in reverse to decrypt the file.

Building and Running

Ensure you have Rust installed: https://rustup.rs/

Clone the repo and run:

cargo run --release
