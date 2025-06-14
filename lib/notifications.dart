import 'package:flutter/material.dart';
import 'package:daily_wallpaper_images/src/bindings/bindings.dart';

class NotificationBubble extends StatelessWidget {
  final NotificationAlert alert;
  final Function()? onTap;

  const NotificationBubble(
    this.alert, {
    super.key,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    var finished = alert.percent >= 1.0;
    final fontColor = switch (alert.severity) {
      NotificationSeverity.warning => Colors.black,
      _ => Colors.white,
    };
    final tileColor = switch (alert.severity) {
      NotificationSeverity.debug => Colors.purple,
      NotificationSeverity.info => Colors.green,
      NotificationSeverity.warning => Colors.orange,
      NotificationSeverity.error => Colors.red,
    };

    final textTheme = Theme.of(context).textTheme;
    return Padding(
      padding: const EdgeInsets.all(8.0),
      child: ListTile(
        title: Text(
          alert.title,
          style: textTheme.labelLarge?.copyWith(color: fontColor),
        ),
        subtitle: Text(
          alert.body,
          style: textTheme.labelMedium?.copyWith(color: fontColor),
        ),
        splashColor: tileColor,
        onTap: onTap,
        contentPadding: EdgeInsets.fromLTRB(16.0, 0, 16.0, 0),
        tileColor: tileColor.withAlpha(135),
        textColor: fontColor,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadiusGeometry.circular(32.0),
        ),
        // trailing: finished
        //     ? IconButton(onPressed: () {}, icon: Icon(Icons.close))
        //     : null,
        leading: finished
            ? alert.statusMessage.isEmpty
                ? Icon(Icons.check)
                : Text(alert.statusMessage)
            : CircularProgressIndicator(value: alert.percent),
      ),
    );
  }
}

class NotificationCenter extends StatefulWidget {
  const NotificationCenter({super.key});

  @override
  State<NotificationCenter> createState() => _NotificationCenterState();
}

class _NotificationCenterState extends State<NotificationCenter> {
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
            : Center(
                child: Column(
                spacing: 10.0,
                children: [
                  CircularProgressIndicator(),
                  Text("Refreshing"),
                ],
              ));
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
