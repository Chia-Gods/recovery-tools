# Chia Gods Recovery Tools

These tools can be used to recover Chia Gods NFT images and metadata from your own local full node.

## Prerequisites

* Synced full node. See [Chia Install Docs](https://docs.chia.net/installation/)
* Rust Toolchain. See [Rust Install Docs](https://rustup.rs/)

## Usage

There are currently four functions of the tool: `locate-nft-data`, `recover-metadata`, `recover-image`, and `recover-collection`.

The first time you run any of the commands may take a while, since it will first need to compile the application.

### Locate NFT Data

The `locate-nft-data` command accepts any NFT ID from the collection and will trace through the parent coins on chain to locate the metadata coin ID and the image coin IDs.
The output of this command can be used in the other commands to restore an image, the full collection, and the metadata files.

`cargo run -- locate-nft-data --nft-id nft1r8cx3ykw4r8x6wkaehd5ye26xfdhzlk7fswz8ctgvc5sj9al3scslv03v6`

### Recover Metadata

The `recover-metadata` command will read the metadata coin and write all metadata files for the whole collection to an `output-metadata` directory. For the Chia Gods collection, the metadata coin ID is `e743335b56ec7428790ba164fe1f130dc7b4bdf32ee16da6f1a09621c27a326c`.

`cargo run -- recover-metadata --coin e743335b56ec7428790ba164fe1f130dc7b4bdf32ee16da6f1a09621c27a326c`

### Recover Image

The `recover-image` command will recover a single image from the collection, given its coin ID and write it to an `output-images` directory. All image coin IDs are referenced in the metadata for the NFTs. The first image in the collection is coin ID `8c0793fece985be90444fa6f01f40861047b3b2307053f378ec72f5a5c4bb4d7`

`cargo run -- recover-image --coin 8c0793fece985be90444fa6f01f40861047b3b2307053f378ec72f5a5c4bb4d7`

### Recover Collection

The `recover-collection` command will recover the images for the entire collection, given the first coin ID in the collection. All images will be written to an `output-images` directory.

`cargo run -- recover-collection --coin 8c0793fece985be90444fa6f01f40861047b3b2307053f378ec72f5a5c4bb4d7`