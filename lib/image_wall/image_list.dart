import 'dart:io';

// import 'package:async_wallpaper/async_wallpaper.dart';
import 'package:daily_wallpaper_images/notifications.dart';
import 'package:flutter/material.dart';
import 'package:daily_wallpaper_images/src/bindings/bindings.dart';
import 'package:shimmer_animation/shimmer_animation.dart';

mixin ImageListPage {
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

  void _pictureModal(BuildContext context, DailyImage img) {
    showDialog<void>(
      context: context,
      builder: (context) {
        var selectedMode = WallpaperMode.fit;
        return AlertDialog(
          title: Text(img.date),
          content: Column(
            children: <Widget>[
              Expanded(
                child: Image.file(
                  File(img.url),
                  fit: BoxFit.contain,
                  semanticLabel: img.description,
                ),
              ),
              Text(img.description),
            ],
          ),
          actions: <Widget>[
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Cancel'),
            ),
            DropdownMenu<WallpaperMode>(
              initialSelection: WallpaperMode.fit,
              label: const Text("Wallpaper Mode"),
              dropdownMenuEntries:
                  WallpaperMode.values.map<DropdownMenuEntry<WallpaperMode>>(
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
            TextButton(
              onPressed: () async {
                if (Platform.isAndroid) {
                  // await AsyncWallpaper.setWallpaperFromFile(
                  //   filePath: img.url,
                  //   wallpaperLocation: AsyncWallpaper.HOME_SCREEN,
                  // );
                } else {
                  SetWallpaper(
                    selected: WallpaperSelection(
                      path: img.url,
                      mode: selectedMode,
                    ),
                  ).sendSignalToRust();
                }
                Navigator.pop(context);
              },
              child: const Text('Set as wallpaper'),
            ),
          ],
        );
      },
    );
  }

  Widget buildLoadingWidget(String imageService) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        spacing: 20.0,
        children: [
          CircularProgressIndicator(),
          Text("Refreshing $imageService")
        ],
      ),
    );
  }

  Widget buildListView(BuildContext context, List<DailyImage> images) {
    var colorScheme = Theme.of(context).colorScheme;

    return Column(
      children: [
        Align(alignment: Alignment.centerRight, child: NotificationsMonitor()),
        Expanded(
          child: GridView.extent(
            padding: EdgeInsets.all(10.0),
            maxCrossAxisExtent: 450.0,
            crossAxisSpacing: 10.0,
            mainAxisSpacing: 10.0,
            children: [
              for (var img in images)
                InkResponse(
                  child: GridTile(
                    footer: Container(
                      color: colorScheme.surface.withAlpha(215),
                      child: Text(
                        img.description,
                        maxLines: 3,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    child: img.url.isEmpty
                        ? Shimmer(
                            interval: const Duration(seconds: 1),
                            duration: const Duration(seconds: 2),
                            color: colorScheme.onSecondaryContainer,
                            child: Container(
                              decoration: BoxDecoration(
                                color: colorScheme.secondaryContainer,
                                shape: BoxShape.rectangle,
                              ),
                            ),
                          )
                        : Image.file(File(img.url)),
                  ),
                  onTap: () {
                    if (img.url.isNotEmpty) {
                      return _pictureModal(context, img);
                    }
                  },
                )
            ],
          ),
        ),
      ],
    );
  }
}
