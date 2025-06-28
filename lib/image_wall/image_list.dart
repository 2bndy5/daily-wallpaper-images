import 'dart:io';

// import 'package:async_wallpaper/async_wallpaper.dart';
import 'package:daily_wallpaper_images/image_wall/image_viewer.dart';
import 'package:daily_wallpaper_images/notifications/pop_ups.dart';
import 'package:flutter/material.dart';
import 'package:daily_wallpaper_images/src/bindings/bindings.dart';
import 'package:shimmer_animation/shimmer_animation.dart';

mixin ImageListPage {
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

    return Stack(
      children: [
        GridView.extent(
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
                    Navigator.push(
                      context,
                      MaterialPageRoute(
                        builder: (context) => ImageViewer(img),
                      ),
                    );
                  }
                },
              )
          ],
        ),
        Align(alignment: Alignment.topRight, child: NotificationsMonitor()),
      ],
    );
  }
}
