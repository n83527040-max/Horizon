import 'dart:async';

import 'package:flutter/services.dart';

import 'actions.dart';
import 'models.dart';

/// The only door between Dart and the Android/Rust host.
///
/// `dispatch` and `snapshot` go over `dev.horizon.android/bridge`; Kotlin then
/// calls the real JNI/C ABI `HorizonEngine` (Rust `flutter_bridge.rs`).
/// `events` carries asynchronous real results (SAF import, game-launch errors,
/// Microsoft sign-in codes) that arrive after a method call has returned.
abstract interface class LauncherBridge {
  Future<LauncherSnapshot> snapshot();
  Future<BridgeResult> dispatch(
    HorizonAction action, {
    Map<String, Object?> args = const <String, Object?>{},
  });
  Future<BridgeResult> cancel();
  /// Starts the real Rust `official_install::install_with_progress` worker.
  Future<BridgeResult> installStart();
  Stream<Map<String, Object?>> get events;
}

class MethodChannelLauncherBridge implements LauncherBridge {
  MethodChannelLauncherBridge([this.channel = const MethodChannel('dev.horizon.android/bridge')]);

  final MethodChannel channel;

  static final StreamController<Map<String, Object?>> _events =
      StreamController<Map<String, Object?>>.broadcast();
  static final Set<MethodChannel> _listening = <MethodChannel>{};

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

  @override
  Future<BridgeResult> cancel() async {
    final Object? reply = await channel.invokeMethod<Object?>('cancel');
    return BridgeResult.fromJson(reply);
  }

  @override
  Future<BridgeResult> installStart() async {
    final Object? reply = await channel.invokeMethod<Object?>('installStart');
    return BridgeResult.fromJson(reply);
  }

  @override
  Stream<Map<String, Object?>> get events {
    if (_listening.add(channel)) {
      channel.setMethodCallHandler((MethodCall call) async {
        if (call.method != 'nativeEvent') return null;
        final Object? arguments = call.arguments;
        if (arguments is Map<Object?, Object?>) {
          _events.add(Map<String, Object?>.from(arguments));
        }
        return null;
      });
    }
    return _events.stream;
  }
}
