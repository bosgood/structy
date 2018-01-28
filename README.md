# structy

Structured logging parser. Accepts input from files and stdin.

## Usage

Run from a file:

```
structy <file.json>
```

Run from stdin:

```
./myservice | structy
```

## Installation

1. Download the latest version from the [Releases page](https://github.com/bosgood/structy/releases)
2. Unzip the archive and place `structy` in your `PATH`.

## Building from source

```
$ cargo build
```

## Running the unit tests

```
$ cargo test
```

## License

[MIT License](https://github.com/bosgood/structy/blob/master/LICENSE). This software is provided as-is, without warranty of any kind.
