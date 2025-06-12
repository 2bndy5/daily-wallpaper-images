# Daily Wallpaper Images

An app to set the device's wallpaper from daily images sources like Bing and Nasa.
This is a free app with no ads using Flutter on the frontend and Rust on the backend.

## Platform support

This app aims to support

- [x] Linux
- [x] Windows
- [ ] Android
- [ ] MacOS

Platforms in the above list with a checkmark have been tested.
This app can build/compile for Android, but it does not run correctly (possibly a TLS certificate problem).
I need a MacOS user(s) to test the MacOS builds.
IOS does not allow programmatically setting the device's wallpaper.

### Linux desktop environments

This was only tested on Ubuntu 24.04 with GNOME desktop.
Theoretically it should work on other Linux desktops as well.
See the supported list of desktop environments in [my fork of wallpaper.rs](https://github.com/2bndy5/wallpaper.rs).

### Windows

On windows, this app seems to have trouble setting the wallpaper for _virtual_ desktops.
Technically, this app should set the wallpaper for all monitors' desktops, but this has not been tested (yet).

## Building from source

### Prerequisites

To build this app, you need to have installed

- [Flutter SDK](https://docs.flutter.dev/get-started/install)
- [Rust toolchain](https://www.rust-lang.org/tools/install)
- [Rinf (Rust-in-Flutter)](https://github.com/cunarist/rinf)

    ```shell
    cargo install rinf_cli
    ```

    Rinf CLI can also be installed with [cargo-binstall](https://github.com/cargo-bins/cargo-binstall).

You can check that your system is ready with the commands below.

```shell
rustc --version
flutter doctor
rinf help
```

Note that all the applicable Flutter sub-components should be installed.
Once the prerequisites are installed, use these commands to run the app:

```shell
git clone --recurse-submodules https://github.com/2bndy5/daily-wallpaper-images.git
cd daily-wallpaper-images
rinf gen
flutter run
```

### Using Rust Inside Flutter

This project leverages Flutter for GUI and Rust for the backend logic,
utilizing the capabilities of the
[Rinf](https://pub.dev/packages/rinf) framework.

Signals sent between Dart and Rust are implemented using signal attributes.
If you've modified the signal structs, run the following command
to generate the corresponding Dart classes:

```shell
rinf gen
```

Now you can run and build this app just like any other Flutter projects.

```shell
flutter run
```

For detailed instructions on writing Rust and Flutter together,
please refer to Rinf's [documentation](https://rinf.cunarist.com).
