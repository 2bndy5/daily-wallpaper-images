import 'dart:io';

import 'package:daily_wallpaper_images/mouse_back_button.dart';
import 'package:daily_wallpaper_images/src/bindings/bindings.dart';
import 'package:flutter/material.dart';

class ImageViewer extends StatefulWidget {
  final DailyImage img;
  const ImageViewer(this.img, {super.key});

  @override
  State<ImageViewer> createState() => _ImageViewerState();
}

class _ImageViewerState extends State<ImageViewer> {
  WallpaperMode? selectedMode;

  Icon _getModeIcon(WallpaperMode mode) {
    return Icon(switch (mode) {
      WallpaperMode.fit => Icons.fit_screen,
      WallpaperMode.stretch => Icons.zoom_out_map,
      WallpaperMode.center => Icons.zoom_in_map,
      WallpaperMode.tile => Icons.grid_view,
      WallpaperMode.crop => Icons.crop,
    });
  }

  String _getModeText(WallpaperMode mode) {
    return mode.name.substring(0, 1).toUpperCase() + mode.name.substring(1);
  }

  @override
  Widget build(BuildContext context) {
    return MouseBackButtonDetector(
      onTapDown: (details) => Navigator.pop(context),
      child: Scaffold(
        appBar: AppBar(
          title: Text(widget.img.date),
          leading: IconButton(
            onPressed: () => Navigator.pop(context),
            icon: Icon(Icons.arrow_back),
          ),
          actions: [
            TextButton.icon(
              icon: Icon(Icons.check),
              onPressed: () async {
                if (Platform.isAndroid) {
                  // await AsyncWallpaper.setWallpaperFromFile(
                  //   filePath: img.url,
                  //   wallpaperLocation: AsyncWallpaper.HOME_SCREEN,
                  // );
                } else {
                  SetWallpaper(
                    selected: WallpaperSelection(
                      path: widget.img.url,
                      mode: selectedMode,
                    ),
                  ).sendSignalToRust();
                }
                Navigator.pop(context);
              },
              label: const Text('Set as wallpaper'),
            ),
            Builder(builder: (context) {
              return IconButton(
                  onPressed: () => Scaffold.of(context).openEndDrawer(),
                  icon: Icon(Icons.display_settings));
            }),
          ],
        ),
        body: Column(
          children: <Widget>[
            Expanded(
              child: Image.file(
                File(widget.img.url),
                fit: BoxFit.contain,
                semanticLabel: widget.img.description,
              ),
            ),
          ],
        ),
        bottomSheet: BottomSheet(
          onClosing: () {},
          builder: (context) => Padding(
            padding: const EdgeInsets.all(8.0),
            child: Text(widget.img.description),
          ),
        ),
        endDrawer: Drawer(
          child: ListView(
            children: [
              DrawerHeader(
                decoration: BoxDecoration(
                    color: Theme.of(context).colorScheme.primaryContainer),
                child: Text("Options"),
              ),
              ListTile(
                title: DropdownMenu<WallpaperMode?>(
                  initialSelection: selectedMode,
                  label: const Text("Wallpaper Mode"),
                  dropdownMenuEntries: [
                        DropdownMenuEntry<WallpaperMode?>(
                          label: "Not Changed",
                          value: null,
                        )
                      ] +
                      WallpaperMode.values
                          .map<DropdownMenuEntry<WallpaperMode?>>(
                        (entry) {
                          return DropdownMenuEntry(
                            label: _getModeText(entry),
                            value: entry,
                            leadingIcon: _getModeIcon(entry),
                          );
                        },
                      ).toList(),
                  onSelected: (value) => {
                    if (value != null) {selectedMode = value}
                  },
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
