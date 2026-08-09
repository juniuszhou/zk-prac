# SP1 Project Template

This is a template for creating an end-to-end [SP1](https://github.com/succinctlabs/sp1) project
that can generate a proof of any RISC-V program.

## Requirements

- [Rust](https://rustup.rs/)
- [SP1](https://docs.succinct.xyz/docs/sp1/getting-started/install)
- [Go](https://go.dev/dl/) 1.21+ (required for `native-gnark` / EVM Groth16-PLONK proofs; needs `go` on `PATH` and `gcc` for CGO)

If `cargo build` fails with `sp1-recursion-gnark-ffi` / `Failed to build Go library: ... NotFound`, install Go and ensure it is on `PATH`:

```sh
# user-local install (no sudo)
curl -fsSL https://go.dev/dl/go1.22.5.linux-amd64.tar.gz -o /tmp/go.tar.gz
rm -rf ~/go && tar -C ~ -xzf /tmp/go.tar.gz
echo 'export PATH="$HOME/go/bin:$PATH"' >> ~/.bashrc
export PATH="$HOME/go/bin:$PATH"
go version
```

If **rust-analyzer** / `cargo check` still cannot find `go` (IDE uses a minimal `PATH`), copy the cargo env config:

```sh
mkdir -p .cargo
cp .cargo/config.toml.example .cargo/config.toml
# edit the go/bin path if needed
```

Or symlink Go into Cargo's bin dir (works when `~/.cargo/bin` is on `PATH`):

```sh
ln -sf "$HOME/go/bin/go" "$HOME/.cargo/bin/go"
```

## Running the Project

There are 3 main ways to run this project: execute a program, generate a core proof, and
generate an EVM-compatible proof.

### Build the Program

The program is automatically built through `script/build.rs` when the script is built.
OR build it with 

```sh
cargo prove build
```

### Execute the Program

To run the program without generating a proof:

```sh
cd script
cargo run --release -- --execute
```

This will execute the program and display the output.

### Generate an SP1 Core Proof

To generate an SP1 [core proof](https://docs.succinct.xyz/docs/next/sp1/generating-proofs/proof-types#core-default) for your program:

```sh
cd script
cargo run --release -- --prove
```

### Generate an EVM-Compatible Proof

> [!WARNING]
> You will need at least 16GB RAM to generate a Groth16 or PLONK proof. View the [SP1 docs](https://docs.succinct.xyz/docs/next/sp1/getting-started/hardware-requirements#local-proving) for more information.

Generating a proof that is cheap to verify on the EVM (e.g. Groth16 or PLONK) is more intensive than generating a core proof.

To generate a Groth16 proof:

```sh
cd script
cargo run --release --bin evm -- --system groth16
```

To generate a PLONK proof:

```sh
cargo run --release --bin evm -- --system plonk
```

These commands will also generate fixtures that can be used to test the verification of SP1 proofs
inside Solidity.

### Retrieve the Verification Key

To retrieve your `programVKey` for your on-chain contract, run the following command in `script`:

```sh
cargo run --release --bin vkey
```

## Using the Prover Network

We highly recommend using the Succinct Prover Network for any non-trivial programs or benchmarking purposes. For more information, see the [quickstart guide](https://docs.succinct.xyz/docs/next/sp1/prover-network/quickstart).

To get started, copy the example environment file:

```sh
cp .env.example .env
```

Then, set the `SP1_PROVER` environment variable to `network` and set the `NETWORK_PRIVATE_KEY`
environment variable to your whitelisted private key.

For example, to generate an EVM-compatible proof using the prover network, run the following
command:

```sh
SP1_PROVER=network NETWORK_PRIVATE_KEY=... cargo run --release --bin evm
```
