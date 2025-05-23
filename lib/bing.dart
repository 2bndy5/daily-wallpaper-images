import 'package:flutter/material.dart';
import 'package:daily_wallpaper_images/src/bindings/bindings.dart';
import 'package:daily_wallpaper_images/image_list.dart';

class BingPage extends StatelessWidget with ImageListPage {
  const BingPage({super.key});

  @override
  Widget build(BuildContext context) {
    BingRefresh().sendSignalToRust();
    return RefreshIndicator(
      onRefresh: () async {
        BingRefresh().sendSignalToRust();
        Future.delayed(Duration(seconds: 2));
        Future(() {});
      },
      child: StreamBuilder(
        stream: BingImageList.rustSignalStream,
        builder: (context, snapshot) {
          if (snapshot.data == null) {
            return buildLoadingWidget("Bing");
          }
          return buildListView(context, snapshot.data!.message.images);
        },
      ),
    );
  }
}
