import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';

class _BackButtonRecognizer extends BaseTapGestureRecognizer {
  GestureTapDownCallback? onTapDown;

  @override
  void handleTapCancel({
    required PointerDownEvent down,
    PointerCancelEvent? cancel,
    required String reason,
  }) {}

  @override
  void handleTapDown({required PointerDownEvent down}) {
    final TapDownDetails details = TapDownDetails(
      globalPosition: down.position,
      localPosition: down.position,
      kind: getKindForPointer(down.pointer),
    );
    if (down.buttons == kBackMouseButton && onTapDown != null) {
      invokeCallback<void>('onTapDown', () => onTapDown!(details));
    }
  }

  @override
  void handleTapUp({
    required PointerDownEvent down,
    required PointerUpEvent up,
  }) {}
}

class MouseBackButtonDetector extends StatelessWidget {
  const MouseBackButtonDetector({
    super.key,
    required this.child,
    this.onTapDown,
  });
  final Widget child;
  final void Function(TapDownDetails details)? onTapDown;

  @override
  Widget build(BuildContext context) {
    return RawGestureDetector(
      excludeFromSemantics: true,
      gestures: <Type, GestureRecognizerFactoryWithHandlers>{
        _BackButtonRecognizer:
            GestureRecognizerFactoryWithHandlers<_BackButtonRecognizer>(
          () => _BackButtonRecognizer(),
          (_BackButtonRecognizer instance) {
            instance.onTapDown = onTapDown ??
                (TapDownDetails details) {
                  Navigator.pop(context);
                };
          },
        ),
      },
      child: child,
    );
  }
}
