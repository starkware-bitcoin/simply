# Simply

Command line tooling for SimplicityHL (previously Simfony), a high-level language for Bitcoin's Simplicity smart contracts.

## Overview

Simply is a comprehensive CLI tool that provides development, testing, and deployment capabilities for SimplicityHL programs. It supports the complete workflow from writing SimplicityHL code to deploying and interacting with Bitcoin smart contracts.

## Installation

```sh
cargo install --git https://github.com/m-kus/simply simply
```

## Commands

### Build

Compiles a SimplicityHL program and optionally generates witness data.

```sh
simply build [OPTIONS]
```

**Flags:**
- `--entrypoint <PATH>` - Path to the source file (default: `./src/main.simf`)
- `--mcpp-inc-path <PATH>` - Path to mcpp include directory (optional, enables C-style preprocessing)
- `--witness <PATH>` - Path to witness file (optional)
- `--prune` - Prune the program using the provided witness (may limit reusability)
- `--target-dir <PATH>` - Output directory for compiled artifacts (default: `./target`)

**Output:** Build artifacts are saved as JSON files containing the compiled program and optional witness data. The build process also displays node bounds and the padding required for your program. Padding represents extra space your program should occupy to compensate for execution resources. Since Bitcoin doesn't have the concept of gas, everything is measured in weight units.

### Run

Executes a SimplicityHL program with optional witness and arguments.

```sh
simply run [OPTIONS]
```

**Flags:**
- All flags from `build` command
- `--param <PATH>` - Path to file containing program arguments (JSON format)
- `--logging <LEVEL>` - Enable debug logging (`info`, `debug`, or `trace`)

**Usage:** Useful for testing programs locally before deployment. By default, the run command uses the same code execution engine as Elements/Liquid nodes, making it ideal for testing compatibility with the actual Bitcoin network. If you specify logging, a Rust runner will be used instead, as it supports debugging features and provides more detailed execution information.

### Test

Automatically discovers and runs test functions in SimplicityHL files.

```sh
simply test [OPTIONS]
```

**Flags:**
- All flags from `build` command
- `--logging <LEVEL>` - Enable debug logging for test execution

**Test Discovery:** Finds all `*.simf` files recursively and executes functions named `test_*`.

### Deposit

Generates a P2TR (Pay-to-Taproot) address for making deposits to a Simplicity program.

```sh
simply deposit [OPTIONS]
```

**Flags:**
- All flags from `build` command

**Output:** Prints a Bitcoin P2TR address that can receive funds for the compiled program. The generated address is a script-only taproot address that uses an unspendable NUMA key, ensuring the funds can only be spent through the Simplicity program logic.

### Withdraw

Spends a transaction output using a Simplicity program.

```sh
simply withdraw [OPTIONS]
```

**Flags:**
- All flags from `build` command
- `--txid <TXID>` - Transaction ID to spend (required)
- `--destination <ADDRESS>` - Destination address for the withdrawal (required)
- `--dry-run` - Generate transaction without broadcasting (prints hex)

**Usage:** Creates and optionally broadcasts a transaction that spends a UTXO using the compiled program.

## File Formats

### Witness Files
JSON files containing witness data for program execution:
```json
{
  "witness": [...]
}
```

### Argument Files
JSON files containing program arguments:
```json
{
  "arguments": [...]
}
```

## Examples

**Basic build:**
```sh
simply build --entrypoint my_program.simf
```

**Build with witness and pruning:**
```sh
simply build --entrypoint main.simf --witness witness.json --prune
```

**Run with arguments:**
```sh
simply run --entrypoint main.simf --param args.json --logging debug
```

**Generate deposit address:**
```sh
simply deposit --entrypoint main.simf
```

**Withdraw funds:**
```sh
simply withdraw --entrypoint main.simf --txid abc123... --destination bc1q...
```

## Resources

- [SimplicityHL Documentation](https://docs.simplicity-lang.org/simplicityhl-reference/)
- [Simplicity Specification](https://blockstream.com/simplicity.pdf)
