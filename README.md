# psx-spu

This crate provides a safe low-level bindings over the PlayStation 1's Sound Processing Unit.

## What this crate can do (as of right now.)

- Iterate through channels.
- Set the frequency and volumes of channels.
- Upload samples to the SPU RAM
- Pitch modulation!

## What this crate can't do (todo)

- ADSR envelopes
- Upload samples using DMA (would use `psx` for this, but their library is having issues being compiled)
- Reverb/Echo and IIR filters
- Maybe some things I've forgotten here.

## License

This crate is licensed under the Apache 2.0 license.
