# qoi-rs

This project is a Rust implementation of the QOI (Quite OK Image) format, designed as a learning exercise to explore idiomatic Rust, iterators, and the `clap` library for building command-line interfaces. This essentially served as a continuation of the `pngme` project tutorial by jrdngr. [https://github.com/jrdngr/pngme]

## Overview

The goal was to implement a fast and efficient QOI parser and encoder using Rust's powerful iterator patterns. The project also includes basic support for PNG and PPM formats, primarily for testing and completeness, with png.rs being a wrapper around the `png` crate.

## Idiomatic Rust and Iterators

The project heavily relies on iterators to process image data. This approach allows for:

- **Zero-Copy Operations**: Working directly with data slices without unnecessary copying.
- **Efficient Data Handling**: Using iterator adaptors like `Peekable` and `Chunks` to manage data streams.
- **Modular Code**: Traits like `Interpret`, `Parse`, `Compress`, and `Assemble` make the code reusable and easy to extend. Care was taken to follow the function chain pattern for code ergonomics.

## Parsing and Encoding

The QOI format is parsed into a stream of chunks by an iterator, which is assembled into a final pixel buffer. Since each chunk may yield a variable number of pixels, the iterator has a &[u8] return type. Encoding raw pixel data into a .qoi image is done in much the same way, with an iterator consuming variable amounts of the byte stream, and assembling it into a stream of chunks.

## Getting Started

To build and run the project:

```bash
# Build the project
cargo build --release

# Run the CLI
./target/release/qoi --help
```

Example commands:

```bash
# Encode a PNG/PPM to QOI
./target/release/qoi convert pics/img.png output/img.qoi
# Any combination of PNG, PPM and QOI work.

# Display a PNG/PPM/QOI
./target/release/qoi open pics/img.qoi output/img.png
```

## Pipes and stdio

Two more commands are omitted due to their limited functionality - 'qoi write' and 'qoi view' work with stdin instead of a file, and take in a dimension-prefixed pixel buffer as input. Write writes to a .ppm/.png/.qoi file, with the possibility to specify -f to write several images from the same stream and -n to number them sequentially. View simply displays what it reads in a winit window. Both were used in conjunction with my `pcls` project to manually record a run of the simulation.

## Conclusion

This project was a valuable learning experience in Rust iterators and building simple command-line tools. As with some other projects, working on this in the absence of internet access allowed me to derive a lot of insights into iterators, and specifically their signatures and working with streams of data in Rust. I'm sure there's a more advanced way of piping data across layers of functionality than assembling u8 slices, but the zero-copy approach has been interesting to implement.
