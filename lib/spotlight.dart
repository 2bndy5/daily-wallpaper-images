import 'package:flutter/material.dart';
import 'package:daily_wallpaper_images/src/bindings/bindings.dart';
import 'package:daily_wallpaper_images/image_list.dart';

class SpotlightPage extends StatelessWidget with ImageListPage {
  const SpotlightPage({super.key});

  @override
  Widget build(BuildContext context) {
    SpotlightRefresh().sendSignalToRust();
    return RefreshIndicator(
      onRefresh: () async {
        SpotlightRefresh().sendSignalToRust();
        Future.delayed(Duration(seconds: 2));
        Future(() {});
      },
      child: StreamBuilder(
        stream: SpotlightImageList.rustSignalStream,
        builder: (context, snapshot) {
          if (snapshot.data == null) {
            return buildLoadingWidget("Windows Spotlight");
          }
          return buildListView(context, snapshot.data!.message.images);
        },
      ),
    );
  }
}
