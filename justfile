_default:
  just -l

# runs rustfmt with nightly to enable all its features
fmt:
  rustup run nightly cargo fmt