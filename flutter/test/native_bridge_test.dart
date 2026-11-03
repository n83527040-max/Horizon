import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:horizon_launcher/core/bridge/actions.dart';
import 'package:horizon_launcher/core/bridge/launcher_bridge.dart';
import 'package:horizon_launcher/core/bridge/models.dart';

/// These tests prove the typed contract reaches real native code. They are
/// source checks + channel tests, NOT device/runtime validation.
void main() {
  const String rustBridge = '../crates/horizon-client/src/android/flutter_bridge.rs';
  const String kotlinBridge =
      '../flutter/android/app/src/main/kotlin/dev/horizon/android/HorizonBridge.kt';
  const String engineKt =
      '../flutter/android/app/src/main/kotlin/dev/horizon/android/HorizonEngine.kt';

  test('BridgeResult exposes real loading and hand-off states', () {
    final BridgeResult pending = BridgeResult.fromJson(const <Object?, Object?>{
      'ok': true,
      'message': 'Downloading',
      'pending': true,
      'changed': true,
    });
    expect(pending.ok, isTrue);
    expect(pending.pending, isTrue);
    expect(pending.changed, isTrue);

    final BridgeResult handoff = BridgeResult.fromJson(const <Object?, Object?>{
      'ok': true,
      'message': 'Starting Minecraft',
      'needsActivity': true,
    });
    expect(handoff.needsActivity, isTrue);
    expect(handoff.pending, isFalse);
  });

  test('snapshot surfaces real pending state and Microsoft device code', () {
    final LauncherSnapshot s = LauncherSnapshot.fromJson(const <Object?, Object?>{
      'appVersion': '0.7.8',
      'gameVersion': '26.3',
      'engineBridgeReady': true,
      'pending': true,
      'loginCode': 'ABCD1234',
      'loginUri': 'https://microsoft.com/devicelogin',
      'loginMessage': 'Waiting for Microsoft approval',
    });
    expect(s.engineBridgeReady, isTrue);
    expect(s.pending, isTrue);
    expect(s.loginCode, 'ABCD1234');
    expect(s.loginUri, 'https://microsoft.com/devicelogin');
    expect(s.loginMessage, 'Waiting for Microsoft approval');
  });

  test('cancel is a real native channel call, not a local no-op', () async {
    const StandardMethodCodec codec = StandardMethodCodec();
    const String channel = 'dev.horizon.android/bridge';
    final TestWidgetsFlutterBinding binding = TestWidgetsFlutterBinding.ensureInitialized();
    final List<MethodCall> calls = <MethodCall>[];
    binding.defaultBinaryMessenger.setMockMessageHandler(channel, (ByteData? message) async {
      final MethodCall call = codec.decodeMethodCall(message);
      calls.add(call);
      return codec.encodeSuccessEnvelope(<String, Object?>{
        'ok': true,
        'message': 'Cancellation requested',
        'changed': false,
      });
    });
    addTearDown(() => binding.defaultBinaryMessenger.setMockMessageHandler(channel, null));

    final LauncherBridge bridge = MethodChannelLauncherBridge();
    final BridgeResult result = await bridge.cancel();
    expect(result.ok, isTrue);
    expect(calls.single.method, 'cancel');
  });

  test('every one of the 49 actions has a real native dispatcher arm', () {
    final String rust = File(rustBridge).readAsStringSync();
    final String kotlin = File(kotlinBridge).readAsStringSync();
    expect(rust.contains('enginePending'), isFalse);
    expect(kotlin.contains('enginePending'), isFalse);
    final Set<String> rustArms = RegExp(r'"([A-Z][A-Za-z0-9_]*)"\s*(?:=>|\|)')
        .allMatches(rust)
        .map((RegExpMatch m) => m.group(1)!)
        .toSet();
    final Set<String> kotlinArms = RegExp(r'"([A-Z][A-Za-z0-9_]*)"\s*->')
        .allMatches(kotlin)
        .map((RegExpMatch m) => m.group(1)!)
        .toSet();
    final List<String> missing = HorizonAction.values
        .map((HorizonAction a) => a.name)
        .where((String a) => !rustArms.contains(a) && !kotlinArms.contains(a))
        .toList();
    expect(missing, isEmpty, reason: 'actions without a real native handler: $missing');
  });

  test('JNI symbols declared in Kotlin all exist in the Rust C ABI', () {
    final String rust = File(rustBridge).readAsStringSync();
    final String kt = File(engineKt).readAsStringSync();
    final Set<String> exported = RegExp(r'Java_dev_horizon_android_HorizonEngine_(\w+)')
        .allMatches(rust)
        .map((RegExpMatch m) => m.group(1)!)
        .toSet();
    final Set<String> declared = RegExp(r'external fun (\w+)\(')
        .allMatches(kt)
        .map((RegExpMatch m) => m.group(1)!)
        .toSet();
    expect(declared.isNotEmpty, isTrue);
    expect(declared.difference(exported), isEmpty);
    expect(exported.difference(declared), isEmpty);
  });
}
