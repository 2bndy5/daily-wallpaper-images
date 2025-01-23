import 'dart:io';

// import 'package:async_wallpaper/async_wallpaper.dart';
import 'package:flutter/material.dart';
import 'package:daily_images/messages/all.dart';
import 'package:shimmer_animation/shimmer_animation.dart';

mixin ImageListPage {
  void _pictureModal(BuildContext context, DailyImage img) {
    showDialog<void>(
      context: context,
      builder: (context) {
        var selectedMode = WallpaperMode.Fit;
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
            DropdownMenu<WallpaperMode>(
              initialSelection: WallpaperMode.Fit,
              dropdownMenuEntries:
                  WallpaperMode.values.map<DropdownMenuEntry<WallpaperMode>>(
                (entry) {
                  return DropdownMenuEntry(
                    label: entry.toString(),
                    value: entry,
                  );
                },
              ).toList(),
              onSelected: (value) => {
                if (value != null) {selectedMode = value}
              },
            ),
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Cancel'),
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
          Text("Refreshing $imageService's Daily Images")
        ],
      ),
    );
  }

  Widget buildListView(BuildContext context, List<DailyImage> images) {
    var colorScheme = Theme.of(context).colorScheme;

    return GridView.extent(
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
    );
  }
}
