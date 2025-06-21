import 'package:flutter/material.dart';
import 'package:daily_wallpaper_images/src/bindings/bindings.dart';

Color getSeverityColor(NotificationSeverity severity) {
  return switch (severity) {
    NotificationSeverity.debug => Colors.purple,
    NotificationSeverity.info => Colors.green,
    NotificationSeverity.warning => Colors.yellow,
    NotificationSeverity.error => Colors.red,
  };
}

Icon getSeverityIcon(NotificationSeverity severity) {
  return Icon(switch (severity) {
    NotificationSeverity.debug => Icons.bug_report,
    NotificationSeverity.info => Icons.check_circle_outline_rounded,
    NotificationSeverity.warning => Icons.warning,
    NotificationSeverity.error => Icons.error,
  });
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
    final noteColor = getSeverityColor(alert.severity);

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
      final finishIcon = getSeverityIcon(alert.severity);
      trailing.addAll([finishIcon, Text(alert.statusMessage)]);
    }

    final surfaceColor = Theme.of(context).colorScheme.surfaceContainerHigh;
    final roundedBorder = RoundedRectangleBorder(
      borderRadius: BorderRadius.circular(16.0),
    );

    return Padding(
      padding: const EdgeInsets.all(8.0),
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: surfaceColor,
          borderRadius: roundedBorder.borderRadius,
          border: BoxBorder.all(
            color: noteColor,
            width: 2,
            style: BorderStyle.solid,
          ),
        ),
        child: ListTile(
          title: Text(alert.title),
          subtitle: Text(alert.body + (alert.body.contains('\n') ? '' : '\n')),
          splashColor: noteColor.withAlpha(126),
          onTap: onTap,
          contentPadding: EdgeInsets.fromLTRB(16.0, 0, 16.0, 0),
          shape: roundedBorder,
          trailing: Column(children: trailing),
        ),
      ),
    );
  }
}
