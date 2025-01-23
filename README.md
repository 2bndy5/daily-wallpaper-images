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
    cargo install rinf
    ```

    Rinf can also be installed with [cargo-binstall](https://github.com/cargo-bins/cargo-binstall).

Once the prerequisites are installed, use these commands to run the app:

```shell
git clone --recurse-submodules https://github.com/2bndy5/daily-wallpaper-images.git
cd daily-wallpaper-images
rinf message
flutter run
```

### Using Rust Inside Flutter

Messages sent between Dart and Rust are implemented using [Protobuf](https://protobuf.dev/programming-guides/proto3/).
If you have newly cloned the project repository
or made changes to the `.proto` files in the `./messages` directory,
run the following command:

```shell
rinf message
```

This will generate dart code in `./lib/messages/` directory, and the
corresponding rust generated code is in `./native/hub/src/messages`.

Now you can run and build this app just like any other Flutter projects.

```bash
flutter run
```

For detailed instructions on writing Rust and Flutter together,
please refer to Rinf's [documentation](https://rinf.cunarist.com).
