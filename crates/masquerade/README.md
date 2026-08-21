# masquerade

Dependency-free validation of media files against their declared type.

```rust
use masquerade::inspect;

let result = inspect("upload.jpg")?;
assert!(result.status().is_valid());
# Ok::<(), std::io::Error>(())
```

The crate performs bounded signature and structural checks without decoding
media content. It accepts filesystem paths, byte slices and arbitrary
`Read + Seek` streams.

`Valid` means that the input passed every check currently implemented for its
declared type. It does not prove that the file is safe to decode, contains no
polyglot payload, or conforms to every requirement of the format specification.
