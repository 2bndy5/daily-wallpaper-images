import 'dart:io';

import 'package:daily_wallpaper_images/mouse_back_button.dart';
import 'package:daily_wallpaper_images/src/bindings/bindings.dart';
import 'package:easy_image_viewer/easy_image_viewer.dart';
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
    WallpaperModeCache().sendSignalToRust();
    return MouseBackButtonDetector(
      onTapDown: (details) => Navigator.pop(context),
      child: StreamBuilder(
          stream: WallpaperModeCache.rustSignalStream,
          builder: (context, asyncSnapshot) {
            if (asyncSnapshot.hasData) {
              selectedMode = asyncSnapshot.data!.message.mode!;
            }
            return Scaffold(
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
                        //   filePath: widget.img.url,
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
                  IconButton(
                    onPressed: () => showDialog(
                      context: context,
                      builder: (context) {
                        return Dialog(
                          child: Padding(
                            padding: const EdgeInsets.all(16.0),
                            child: Text(widget.img.description),
                          ),
                        );
                      },
                    ),
                    icon: Icon(Icons.info_outline),
                  ),
                  Builder(builder: (context) {
                    return IconButton(
                        onPressed: () {
                          Scaffold.of(context).openEndDrawer();
                        },
                        icon: Icon(Icons.display_settings));
                  }),
                ],
              ),
              body: Column(
                children: <Widget>[
                  Expanded(
                    child: EasyImageView.imageWidget(
                      Image.file(
                        File(widget.img.url),
                        fit: BoxFit.contain,
                        semanticLabel: widget.img.description,
                      ),
                      doubleTapZoomable: true,
                    ),
                  ),
                ],
              ),
              endDrawer: Drawer(
                child: ListView(
                  children: [
                    DrawerHeader(
                      decoration: BoxDecoration(
                          color:
                              Theme.of(context).colorScheme.primaryContainer),
                      child: Text("Options"),
                    ),
                    ListTile(
                      title: DropdownMenu<WallpaperMode?>(
                        initialSelection: selectedMode,
                        label: const Text("Wallpaper Mode"),
                        dropdownMenuEntries: WallpaperMode.values
                            .map<DropdownMenuEntry<WallpaperMode?>>(
                          (entry) {
                            return DropdownMenuEntry(
                              label: _getModeText(entry),
                              value: entry,
                              leadingIcon: _getModeIcon(entry),
                            );
                          },
                        ).toList(),
                        onSelected: (value) {
                          if (value != null) {
                            selectedMode = value;
                            WallpaperModeCache(mode: value).sendSignalToRust();
                          }
                        },
                      ),
                    ),
                  ],
                ),
              ),
            );
          }),
    );
  }
}
