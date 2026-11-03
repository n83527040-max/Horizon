import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:horizon_launcher/app.dart';
import 'package:horizon_launcher/core/bridge/actions.dart';
import 'package:horizon_launcher/core/bridge/launcher_bridge.dart';
import 'package:horizon_launcher/core/bridge/models.dart';
import 'package:horizon_launcher/core/state/providers.dart';

/// Test-only bridge. Production never uses this: the real bridge is a
/// MethodChannel to the Android/Rust host. It only supplies deterministic
/// fixtures so layout/overflow can be measured; no fake values ship in the app.
class FakeBridge implements LauncherBridge {
  FakeBridge(this.fixture);

  final LauncherSnapshot fixture;
  final List<Map<String, Object?>> dispatches = <Map<String, Object?>>[];

  @override
  Future<LauncherSnapshot> snapshot() async => fixture;

  @override
  Future<BridgeResult> dispatch(
    HorizonAction action, {
    Map<String, Object?> args = const <String, Object?>{},
  }) async {
    dispatches.add(<String, Object?>{'action': action.name, 'args': args});
    return const BridgeResult(true, 'Applied', changed: true);
  }
}

LauncherSnapshot fixture({
  bool rich = true,
  String theme = 'dark',
  String language = 'en',
}) {
  return LauncherSnapshot(
    appVersion: '0.7.6',
    gameVersion: '26.3',
    profileName: rich ? 'Player' : null,
    accountStatus: rich ? 'Offline: Player' : null,
    graphicsApi: rich ? 'Vulkan / compatibility' : null,
    renderScalePercent: rich ? 100 : null,
    ramUsedMb: rich ? 267 : null,
    ramPeakMb: rich ? 333 : null,
    ramTotalMb: rich ? 3789 : null,
    installed: rich
        ? <InstalledItem>[
            const InstalledItem(file: 'zoom.hmod', title: 'Zoom', kind: ContentKind.Mods, enabled: true, index: 0),
            const InstalledItem(file: 'fresh.zip', title: 'Fresh Animations', kind: ContentKind.Packs, enabled: false, index: 1),
            const InstalledItem(file: 'bsl.zip', title: 'BSL Shaders', kind: ContentKind.Shaders, index: 2),
          ]
        : const <InstalledItem>[],
    results: rich
        ? <CatalogProject>[
            const CatalogProject(
              title: 'A very long resource pack title used to stress narrow screens without overflowing',
              description: 'A description that is deliberately long enough to wrap on a 360dp phone and must be clipped safely.',
              version: '26.3',
              source: 'Modrinth',
              compatibility: 'Works',
              index: 0,
            ),
          ]
        : const <CatalogProject>[],
    status: rich ? '1 result - select Install to check compatibility' : null,
    settings: rich
        ? <String, Object?>{
            SettingKeys.renderer: 'Vulkan / compatibility',
            SettingKeys.renderScale: 100,
            SettingKeys.orientation: 'portrait',
            SettingKeys.immersive: true,
            SettingKeys.avoidCutout: false,
            SettingKeys.performanceHints: true,
            SettingKeys.gpuProfile: false,
            SettingKeys.recordPerformance: false,
            SettingKeys.theme: theme,
            SettingKeys.language: language,
            SettingKeys.server: '',
            SettingKeys.profile: '0',
          }
        : <String, Object?>{
            SettingKeys.theme: theme,
            SettingKeys.language: language,
          },
    choices: const <String, List<String>>{
      'Renderer': <String>['Vulkan / batching', 'Vulkan / compatibility', 'OpenGL ES / experimental'],
    },
    storageBytes: rich ? 123456789 : null,
    cacheBytes: rich ? 45678901 : null,
    contentBytes: const <String, int>{'mods': 1024000, 'packs': 2048000, 'shaders': 512000},
  );
}

Future<void> pumpApp(
  WidgetTester tester,
  LauncherSnapshot data, {
  Size size = const Size(393, 873),
  FakeBridge? bridge,
}) async {
  tester.view.physicalSize = size;
  tester.view.devicePixelRatio = 1.0;
  addTearDown(() {
    tester.view.resetPhysicalSize();
    tester.view.resetDevicePixelRatio();
  });
  final FakeBridge fake = bridge ?? FakeBridge(data);
  await tester.pumpWidget(
    ProviderScope(
      overrides: <Override>[
        bridgeProvider.overrideWithValue(fake),
        snapshotProvider.overrideWith((Ref ref) => Stream<LauncherSnapshot>.value(data)),
      ],
      child: const HorizonApp(),
    ),
  );
  await tester.pump();
  await tester.pump();
}
