# MXC

Music x computator/calculator

It compute music:

- [x] ReplayGain (2)
- [x] DR Meter
- [ ] Checksum (FLAC only)
- [ ] BPM
- [ ] Chromaprint (fingerprint)
- [ ] Path & filename structure

## Install

Make sure you have Rust installed. Then install Just a command runner with: `cargo install just` and than run `just install`

### Options

- (Default) for TagLib 1.12.0 pass `--features "taglib112"`
- for system TagLib pass `--no-default-features`
- for latest TagLib pass `--features "taglib1xx"`

## Usage

## Other programs in tree

### Loudgain

Cargo package that allows to install loudgain using custom TagLib version.

### loudgainer

Loudgain compatible program written in Rust.

### Loudgain_tester

Testing loudgainer against loudgain for compatibility.
