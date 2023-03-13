# shiv

## 📋 Usage

<!-- help start -->

```
$ shiv --help
Shiv: shell access everywhere.

Shiv allows you to run shell commands from any text box.
When started, it listens for keyboard inputs, on Enter it will run the command and write the output.

The recommended way to use shiv is to bind it to a key combination.

Examples:
  • On demand python shell:
    $ shiv "python -c"
  • Query ChatGPT:
    $ shiv "sgpt"
  • On demand calculator and consersions:
    $ shiv "qalc -t"
  • ASCII art:
    $ shiv "figlet"

Usage: shiv [OPTIONS] [PRE_CMD]

Arguments:
  [PRE_CMD]
          Prefix input with this command
          
          [default: "bash -c"]

Options:
  -T, --type-output
          Type out the command output instead of pasting it

  -d, --key-delay <KEY_DELAY>
          Add delay between keypresses, in ms, values between 0 and 10 work best

  -v, --verbose...
          Increase verbosity

  -q, --quiet...
          Decrease verbosity

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Note: shiv requires priviledges to create and access keyboard devices.
```

<!-- help end -->

## ✔️ TODO:

- [x] add a cli
- [ ] add tests
- [ ] add ci
- [ ] look into better permission handling
- [ ] test on different layouts
- [ ] Run command in separate thread, and still allow user to cancel running command.
- [ ] Give some feedback that the command is running
