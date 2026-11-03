import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:horizon_launcher/core/bridge/actions.dart';
import 'package:horizon_launcher/core/bridge/launcher_bridge.dart';
import 'package:horizon_launcher/core/bridge/models.dart';

void main() {
    final TestWidgetsFlutterBinding binding = TestWidgetsFlutterBinding.ensureInitialized();

  test('MethodChannel bridge sends the exact native action name', () async {
    final List<MethodCall> calls = <MethodCall>[];
    binding.defaultBinaryMessenger
        .setMockMethodChannelHandler('dev.horizon.android/bridge', (MethodCall call) async {
      calls.add(call);
      if (call.method == 'snapshot') {
        return <String, Object?>{
          'appVersion': '0.7.6',
          'gameVersion': '26.3',
          'installed': <Object?>[],
          'results': <Object?>[],
        };
      }
      return <String, Object?>{'ok': true, 'message': 'Applied', 'changed': true};
    });

    const LauncherBridge bridge = MethodChannelLauncherBridge();
    final LauncherSnapshot snapshot = await bridge.snapshot();
    expect(snapshot.appVersion, '0.7.6');
    expect(snapshot.gameVersion, '26.3');
    expect(snapshot.ramUsedMb, isNull, reason: 'no fake RAM when the host omits it');
    expect(snapshot.installed, isEmpty);

    final BridgeResult result = await bridge.dispatch(
      HorizonAction.Install,
      args: const <String, Object?>{'index': 3, 'kind': 'Mods'},
    );
    expect(result.ok, isTrue);
    expect(result.changed, isTrue);
    expect(calls.last.method, 'dispatch');
    expect((calls.last.arguments as Map<Object?, Object?>)['action'], 'Install');
  });

  test('snapshot parsing keeps every field nullable/empty by default', () {
    final LauncherSnapshot s = LauncherSnapshot.fromJson(const <Object?, Object?>{
      'appVersion': '0.7.6',
      'gameVersion': '26.3',
    });
    expect(s.graphicsApi, isNull);
    expect(s.renderScalePercent, isNull);
    expect(s.ramUsedMb, isNull);
    expect(s.avatarPath, isNull);
    expect(s.choices, isEmpty);
    expect(s.settings, isEmpty);
    expect(s.totalInstalled, 0);
    expect(s.count(ContentKind.Shaders), 0);
  });

  test('snapshot parsing maps real installed items and catalog projects', () {
    final LauncherSnapshot s = LauncherSnapshot.fromJson(<Object?, Object?>{
      'appVersion': '0.7.6',
      'gameVersion': '26.3',
      'installed': <Object?>[
        <Object?, Object?>{
          'file': 'zoom.hmod',
          'title': 'Zoom',
          'folder': 'horizon-mods',
          'enabled': true,
          'index': 0,
        },
      ],
      'results': <Object?>[
        <Object?, Object?>{'title': 'Minimap', 'version': '26.3', 'index': 1},
      ],
    });
    expect(s.totalInstalled, 1);
    expect(s.installed.first.kind, ContentKind.Mods);
    expect(s.results.first.title, 'Minimap');
  });
}
