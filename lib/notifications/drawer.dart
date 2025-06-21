import 'package:flutter/material.dart';
import 'package:daily_wallpaper_images/src/bindings/bindings.dart';
import 'package:daily_wallpaper_images/notifications/bubble.dart';

class NotificationCenter extends StatelessWidget {
  const NotificationCenter({super.key});

  @override
  Widget build(BuildContext context) {
    NotificationRefresh().sendSignalToRust();
    return StreamBuilder(
      stream: NotificationResults.rustSignalStream,
      builder: (context, snapshot) {
        var child = snapshot.hasData
            ? (snapshot.data!.message.notifications.isNotEmpty
                ? ListView(
                    children: List.from(
                      snapshot.data!.message.notifications.entries
                          .map((e) => NotificationBubble(
                                e.value,
                                onTap: () =>
                                    NotificationDismiss(timestamp: e.key)
                                        .sendSignalToRust(),
                              )),
                    ),
                  )
                : Center(
                    child: Text("No new notifications to brag about"),
                  ))
            : Align(
                child: Center(
                    child: Column(
                  spacing: 10.0,
                  children: [
                    CircularProgressIndicator(),
                    Text("Refreshing"),
                  ],
                )),
              );
        return Drawer(
          child: Column(
            children: <Widget>[
              DrawerHeader(
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.primaryContainer,
                ),
                child: Column(
                  children: [
                    Row(
                      children: [
                        IconButton(
                          onPressed: () {
                            Navigator.pop(context);
                          },
                          icon: Icon(Icons.close),
                        ),
                        Expanded(child: Container()),
                        IconButton(
                          onPressed: () {
                            NotificationDismissAll().sendSignalToRust();
                            Navigator.pop(context);
                          },
                          icon: Icon(Icons.clear_all),
                        ),
                      ],
                    ),
                    Expanded(
                      child: Align(
                        alignment: AlignmentDirectional.bottomStart,
                        child: Text("Notifications for Nerds"),
                      ),
                    ),
                  ],
                ),
              ),
              Expanded(child: child),
            ],
          ),
        );
      },
    );
  }
}
