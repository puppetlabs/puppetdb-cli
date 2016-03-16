# rust-puppet-access

Rust puppet-access bindings which depend on C bindings defined in
[an experimental branch of puppet-access](https://github.com/ajroetker/puppet-access/tree/experiment/c_bindings_for_important_consumer_functions).

## Example

On OSX with the correct branch of puppet-access installed to `/usr/local/lib`

```
LIBRARY_PATH=/usr/local/lib cargo build
```

Or a sample `build.rs` to make your life easier.

```rust
// build.rs
fn main() {
    println!("cargo:rustc-link-search=native=/usr/local/lib");
    println!("cargo:rustc-link-lib=puppet-access");
}
```

