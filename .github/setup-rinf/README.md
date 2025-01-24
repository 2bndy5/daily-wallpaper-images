# setup-rinf

A composite action to setup Rinf (Rust inside Flutter) environment.

## Supported platforms

This action can setup a build environment used to target the following platforms:

- `android` (can use any OS runner)
- `web` (can use any OS runner)
- `linux` (requires Debian-based Linux runner)
- `windows` (requires Windows-based runner)
- `macos` (requires MacOS-based runner)
- `ios` (requires MacOS-based runner)

### Simple Example

```yml
- uses: ./.github/setup-rinf
  with:
    platform: android # required
```

## Caching

Caching can be enabled for pub.dev dependencies, cargo dependencies, and even the Flutter SDK.
However, cache keys are required for caching pub.dev and cargo dependencies.

By default, caching is disabled.

### Cache Example

```yml
- uses: ./.github/setup-rinf
  with:
    platform: android # required
    cache-cargo: true
    cache-cargo-key: ${{ hashFiles('Cargo.lock') }}-android-cargo
    cache-pub: true
    cache-pub-key: ${{ hashFiles('pubspec.lock') }}-android-dart
    cache-flutter-sdk: true # no cache key is needed
    # cache key for Flutter SDK is handled by `flutter-actions/setup-flutter` internally
```

## Outputs

This action creates useful outputs about the installed versions of the

- Flutter SDK (`flutter-version`)
- Dart SDK (`dart-version`)
- Rust version (`rust-version`)
