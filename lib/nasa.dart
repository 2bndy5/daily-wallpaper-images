import 'package:flutter/material.dart';
import 'package:daily_wallpaper_images/messages/all.dart';
import 'package:daily_wallpaper_images/image_list.dart';

class NasaPage extends StatelessWidget with ImageListPage {
  NasaPage({super.key});

  @override
  Widget build(BuildContext context) {
    NasaRefresh().sendSignalToRust();
    return RefreshIndicator(
      onRefresh: () async {
        NasaRefresh().sendSignalToRust();
        Future.delayed(Duration(seconds: 2));
        Future(() {});
      },
      child: StreamBuilder(
        stream: NasaImageList.rustSignalStream,
        builder: (context, snapshot) {
          if (snapshot.data == null) {
            return buildLoadingWidget("Nasa");
          }
          return buildListView(context, snapshot.data!.message.images);
        },
      ),
    );
  }
}
