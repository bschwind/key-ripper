# key-ripper

## Dependencies

```
rustup target add thumbv6m-none-eabi
cargo install elf2uf2-rs
```

## Flash Code

```
cargo run --release
```

### Troubleshooting

If you get an error such as:

```
Error: "Memory segment 0x010000->0x010094 is outside of valid address range for device"
```

Double check that your `RUSTFLAGS` environment variable, as it will take precedence over the values set in `./cargo/config.toml`.
