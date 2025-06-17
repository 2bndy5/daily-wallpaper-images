import 'package:flutter/material.dart';
import 'package:daily_wallpaper_images/src/bindings/bindings.dart';
import 'package:daily_wallpaper_images/image_wall/image_list.dart';

String getServiceName(ImageService service) {
  switch (service) {
    case ImageService.bing:
      return "Bing Images of the Day";
    case ImageService.nasa:
      return "NASA Images of the Day";
    case ImageService.spotlight:
      return "Windows Spotlight Images";
  }
}

class ImageWall extends StatelessWidget with ImageListPage {
  final ImageService service;
  const ImageWall({super.key, required this.service});

  @override
  Widget build(BuildContext context) {
    Refresh(value: service).sendSignalToRust();
    return RefreshIndicator(
      onRefresh: () async {
        Refresh(value: service).sendSignalToRust();
        Future.delayed(Duration(seconds: 2));
        Future(() {});
      },
      child: StreamBuilder(
        stream: ImageList.rustSignalStream,
        builder: (context, snapshot) {
          if (snapshot.data == null) {
            return buildLoadingWidget(getServiceName(service));
          }
          return buildListView(context, snapshot.data!.message.images);
        },
      ),
    );
  }
}
