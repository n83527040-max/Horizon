import 'package:flutter/services.dart';

import 'actions.dart';
import 'models.dart';

/// The only door between Dart and the Android/Rust host.
/// The native side is `dev.horizon.android/bridge` (MethodChannel), implemented
/// in `flutter/android/app/src/main/kotlin/.../HorizonBridge.kt`.
abstract interface class LauncherBridge {
  Future<LauncherSnapshot> snapshot();
  Future<BridgeResult> dispatch(
    HorizonAction action, {
    Map<String, Object?> args = const <String, Object?>{},
  });
}

class MethodChannelLauncherBridge implements LauncherBridge {
  const MethodChannelLauncherBridge([this.channel = const MethodChannel('dev.horizon.android/bridge')]);

  final MethodChannel channel;

  @override
  Future<LauncherSnapshot> snapshot() async {
    final Object? reply = await channel.invokeMethod<Object?>('snapshot');
    if (reply is! Map<Object?, Object?>) {
      throw const FormatException('Native host returned no launcher snapshot');
    }
    return LauncherSnapshot.fromJson(reply);
  }

  @override
  Future<BridgeResult> dispatch(
    HorizonAction action, {
    Map<String, Object?> args = const <String, Object?>{},
  }) async {
    final Object? reply = await channel.invokeMethod<Object?>(
      'dispatch',
      <String, Object?>{'action': action.name, 'args': args},
    );
    return BridgeResult.fromJson(reply);
  }
}
