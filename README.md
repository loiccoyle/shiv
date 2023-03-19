# shiv

`shiv` is a jankier version of [`shin`](https://github.com/p-e-w/shin), which does not require `ibus`.

It allows you to run shell commands from any text box.

## ❓ How

When started, `shiv` grabs the user's keyboard and creates a virtual pass-through keyboard.

It keeps track of the user's inputs and forwards them selectively to the virtual keyboard.

When the enter key is pressed, the provided command is run and its output is pasted in (or typed out using the `-T` option).

> **Note:** To create and grab the keyboard devices `shiv` needs elevated privileges. The command, however, is run as the invoking user.

## 📋 Usage

`shiv` is designed to be bound to a key combination.

For example, I use the following [`sxhkd`](https://github.com/baskerville/sxhkd) mappings:

```
super + i; any + g
    sudo shiv -d 5 "sgpt"
super + i; any + f
    sudo shiv -d 5 "figlet"
super + i; any + i
    sudo shiv -d 5
```

As always, if in doubt, see the `--help`:

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
          Add delay between keypresses, in ms, values between 1 and 10 work best

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
- [x] add tests
- [x] add ci
- [x] Run command in separate thread, and still allow user to cancel running command.
- [ ] look into better permission handling
- [ ] test on different layouts
- [ ] Give some feedback that the command is running
