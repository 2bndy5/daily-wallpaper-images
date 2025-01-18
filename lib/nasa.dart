import 'dart:io';

// import 'package:async_wallpaper/async_wallpaper.dart';
import 'package:flutter/material.dart';
import 'package:daily_images/messages/all.dart';

class NasaPage extends StatelessWidget {
  const NasaPage({super.key});

  Future<void> refreshNasaImages() async {
    NasaRefresh().sendSignalToRust();
    Future.delayed(Duration(seconds: 2));
    Future(() {});
  }

  void _pictureModal(BuildContext context, NasaImage img) {
    showDialog<void>(
      context: context,
      builder: (context) {
        return AlertDialog(
          title: Text(img.date),
          content: Center(
            child: Container(
              constraints: BoxConstraints(minHeight: 200.0),
              child: Image.file(
                File(img.url),
                fit: BoxFit.contain,
              ),
            ),
          ),
          actions: <Widget>[
            Text(img.description),
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
                      mode: WallpaperMode.Fit,
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

  @override
  Widget build(BuildContext context) {
    NasaRefresh().sendSignalToRust();
    return RefreshIndicator(
      onRefresh: refreshNasaImages,
      child: StreamBuilder(
        stream: NasaImageList.rustSignalStream,
        builder: (context, snapshot) {
          if (snapshot.data == null) {
            return Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                spacing: 20.0,
                children: [
                  CircularProgressIndicator(),
                  Text("Refreshing Nasa's Daily Images")
                ],
              ),
            );
          }
          return GridView.extent(
            padding: EdgeInsets.all(10.0),
            maxCrossAxisExtent: 450.0,
            crossAxisSpacing: 10.0,
            mainAxisSpacing: 10.0,
            children: [
              for (var img in snapshot.data!.message.images)
                InkResponse(
                  child: GridTile(
                    footer: Container(
                      color:
                          Theme.of(context).colorScheme.surface.withAlpha(215),
                      child: Text(
                        img.description,
                        maxLines: 3,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    child: Image.file(File(img.url)),
                  ),
                  onTap: () {
                    return _pictureModal(context, img);
                  },
                )
            ],
          );
        },
      ),
    );
  }
}
