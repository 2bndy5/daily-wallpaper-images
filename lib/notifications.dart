import 'package:flutter/material.dart';
import 'package:daily_wallpaper_images/src/bindings/bindings.dart';

Color _noteColor(NotificationSeverity severity) {
  return switch (severity) {
    NotificationSeverity.debug => Colors.purple,
    NotificationSeverity.info => Colors.green,
    NotificationSeverity.warning => Colors.yellow,
    NotificationSeverity.error => Colors.red,
  };
}

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
    final tileColor = _noteColor(alert.severity);

    var trailing = <Widget>[];
    if (alert.percent < 1.0) {
      trailing.add(
        Stack(
          children: [
            CircularProgressIndicator(
              value: alert.percent > 0 ? alert.percent : null,
            ),
            Positioned(
              width: 36.0,
              height: 36.0,
              child: Align(
                child: Center(
                  child: Text("${(alert.percent * 100).floor()}%"),
                ),
              ),
            ),
          ],
        ),
      );
    } else {
      final finishIcon = Icon(switch (alert.severity) {
        NotificationSeverity.debug => Icons.bug_report,
        NotificationSeverity.info => Icons.check_circle_outline_rounded,
        NotificationSeverity.warning => Icons.warning,
        NotificationSeverity.error => Icons.error,
      });
      trailing.addAll([finishIcon, Text(alert.statusMessage)]);
    }

    return Padding(
      padding: const EdgeInsets.all(8.0),
      child: ListTile(
        title: Text(alert.title),
        subtitle: Text(alert.body),
        splashColor: tileColor.withAlpha(85),
        onTap: onTap,
        contentPadding: EdgeInsets.fromLTRB(16.0, 0, 16.0, 0),
        tileColor: tileColor.withAlpha(23),
        shape: RoundedRectangleBorder(
          side: BorderSide(color: tileColor),
          borderRadius: BorderRadiusGeometry.circular(16.0),
        ),
        trailing: Column(children: trailing),
      ),
    );
  }
}

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

class NotificationsMonitor extends StatelessWidget {
  const NotificationsMonitor({super.key});

  @override
  Widget build(BuildContext context) {
    return StreamBuilder(
      stream: NotificationResults.rustSignalStream,
      builder: (context, snapshot) {
        List<String>? pending = snapshot.data?.message.pending;
        Map<String, NotificationAlert>? data =
            snapshot.data?.message.notifications;
        final popUp = pending != null && pending.isNotEmpty && snapshot.hasData
            ? SizedBox(
                height: 80,
                child: ListView(
                  children: List.from(
                    pending.map(
                      (entry) => NotificationBubble(
                        data![entry]!,
                        onTap: () => NotificationDismiss(
                          timestamp: entry,
                        ).sendSignalToRust(),
                      ),
                    ),
                  ),
                ),
              )
            : Container();
        return popUp;
      },
    );
  }
}
