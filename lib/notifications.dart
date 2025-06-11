import 'package:flutter/material.dart';
import 'package:daily_wallpaper_images/src/bindings/bindings.dart';

class NotificationBubble extends StatelessWidget {
  final NotificationAlert alert;

  const NotificationBubble(
    this.alert, {
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    var finished = alert.percent != null && alert.percent! >= 100.0;
    String? elapsedDescription;
    final fontColor = switch (alert.severity) {
      NotificationSeverity.warning => Colors.black,
      _ => Colors.white,
    };
    return ListTile(
      title: Text(
        alert.title,
        style: Theme.of(context)
            .textTheme
            .headlineSmall!
            .copyWith(color: fontColor),
      ),
      subtitle: Text(
        alert.body,
        style:
            Theme.of(context).textTheme.labelMedium!.copyWith(color: fontColor),
      ),
      splashColor: switch (alert.severity) {
        NotificationSeverity.debug => Colors.purple,
        NotificationSeverity.info => Colors.blue,
        NotificationSeverity.warning => Colors.orange,
        NotificationSeverity.error => Colors.red,
      },
      trailing: Row(
        children: [
          if (finished) IconButton(onPressed: () {}, icon: Icon(Icons.close)),
        ],
      ),
      leading: finished
          ? Text(elapsedDescription!)
          : (alert.percent != null
              ? CircularProgressIndicator(value: alert.percent!)
              : null),
    );
  }
}

class NotificationCenter {
  List<NotificationAlert> notifications = [];
  NotificationCenter();
  void update(NotificationAlert note) {
    var done = false;
    for (var i = 0; i < notifications.length; ++i) {
      if (notifications[i].title == note.title &&
          notifications[i].severity == note.severity) {
        notifications[i] = note;
        done = true;
        break;
      }
    }
    if (!done) {
      notifications.add(note);
    }
  }

  List<NotificationBubble> getNotifications() {
    return List.from(notifications.map((note) => NotificationBubble(note)));
  }
}
