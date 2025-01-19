# daily_images

A new Flutter project.

## Building from source

To build this app from git source, [`rinf`](https://github.com/cunarist/rinf) must first be installed.

```text
cargo install rinf
```

Rinf can also be installed with [cargo-binstall](https://github.com/cargo-bins/cargo-binstall).

Now, use these commands to run the app:

```text
git clone --recurse-submodules https://gitlab.com/2bndy5/daily_images.git
cd daily_images
rinf message
flutter run
```

## Using Rust Inside Flutter

This project leverages Flutter for GUI and Rust for the backend logic,
utilizing the capabilities of the
[Rinf](https://pub.dev/packages/rinf) framework.

To run and build this app, you need to have
[Flutter SDK](https://docs.flutter.dev/get-started/install)
and [Rust toolchain](https://www.rust-lang.org/tools/install)
installed on your system.
You can check that your system is ready with the commands below.
Note that all the Flutter subcomponents should be installed.

```bash
rustc --version
flutter doctor
```

You also need to have the CLI tool for Rinf ready.

```bash
cargo install rinf
```

Messages sent between Dart and Rust are implemented using Protobuf.
If you have newly cloned the project repository
or made changes to the `.proto` files in the `./messages` directory,
run the following command:

```bash
rinf message
```

Now you can run and build this app just like any other Flutter projects.

```bash
flutter run
```

For detailed instructions on writing Rust and Flutter together,
please refer to Rinf's [documentation](https://rinf.cunarist.com).
