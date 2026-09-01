# Single file transfer

A small Rust snippet from a directory sync client.

It shows two things:

+ **Resume** - the file is sent in packets. Each packet has a position and a hash. The server replies with how far it got. If the transfer stops, the next try starts at that point.
+ **Hash check**  - when the last packet is in, the server hashes the whole file and compares it with the client hash.
