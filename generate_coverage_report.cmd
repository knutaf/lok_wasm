setlocal
set RUSTFLAGS=%RUSTFLAGS% -C instrument-coverage
del default_*.prof*
cargo test
cargo profdata -- merge -sparse default_*.profraw -o default.profdata
cargo cov -- show -Xdemangler=rustfilt target\debug\deps\lok_wasm-4d4ce0b412053771.exe --instr-profile=default.profdata --output-dir=coverage --format=html
