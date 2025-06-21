import 'package:flutter/material.dart';
import 'package:daily_wallpaper_images/src/bindings/bindings.dart';
import 'bubble.dart';

class NotificationsMonitor extends StatefulWidget {
  const NotificationsMonitor({super.key});

  @override
  State<NotificationsMonitor> createState() => _NotificationsMonitorState();
}

class _NotificationsMonitorState extends State<NotificationsMonitor>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    duration: const Duration(milliseconds: 1500),
    vsync: this,
  );
  late final Animation<Offset> _offsetAnimation = Tween<Offset>(
    begin: Offset.zero,
    end: const Offset(1.5, 0.0),
  ).animate(
    CurvedAnimation(
      parent: _controller,
      curve: Interval(0.66, 1.0, curve: Curves.easeIn),
    ),
  );

  @override
  Widget build(BuildContext context) {
    return StreamBuilder(
      stream: NotificationResults.rustSignalStream,
      builder: (context, snapshot) {
        List<String>? pending = snapshot.data?.message.pending;
        List<String>? justFinished = snapshot.data?.message.justFinished;
        Map<String, NotificationAlert>? data =
            snapshot.data?.message.notifications;
        var children = <Widget>[];

        if (snapshot.hasData) {
          _controller.value = 0;
          _controller.stop(canceled: true);
          if (justFinished!.isNotEmpty) {
            for (final item in justFinished) {
              children.add(
                SlideTransition(
                  position: _offsetAnimation,
                  child: NotificationBubble(data![item]!),
                ),
              );
            }
          }
          if (pending!.isNotEmpty) {
            for (final item in pending) {
              children.add(NotificationBubble(data![item]!));
            }
          }
        }
        if (children.isNotEmpty) {
          _controller.forward();
        }
        final popUps = children.isNotEmpty
            ? ConstrainedBox(
                constraints: BoxConstraints(
                  maxWidth: Theme.of(context).drawerTheme.width ?? 450,
                ),
                child: Material(
                  type: MaterialType.transparency,
                  elevation: 1.0,
                  borderOnForeground: false,
                  child: ListView(
                    shrinkWrap: true,
                    children: children,
                  ),
                ),
              )
            : Container();
        return popUps;
      },
    );
  }
}
