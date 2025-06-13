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
    final fontColor = switch (alert.severity) {
      NotificationSeverity.warning => Colors.black,
      _ => Colors.white,
    };
    final tileColor = switch (alert.severity) {
      NotificationSeverity.debug => Colors.purple,
      NotificationSeverity.info => Colors.blue,
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
        onTap: () {},
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
            ? Text(alert.statusMessage)
            : CircularProgressIndicator(value: alert.percent),
      ),
    );
  }
}
