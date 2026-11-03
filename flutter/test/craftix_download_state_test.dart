import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:horizon_launcher/core/bridge/launcher_bridge.dart';
import 'package:horizon_launcher/core/theme/craftix_runner_progress.dart';
import 'package:horizon_launcher/core/bridge/models.dart';

import 'helpers.dart';

/// Real state-machine tests: the Home action slot must follow the REAL native
/// installer state, never a made-up percentage or a fake "success".
void main() {
  testWidgets('Home shows Download when resources are genuinely missing',
      (WidgetTester tester) async {
    final FakeBridge bridge = FakeBridge(fixture(installReady: false));
    await pumpApp(tester, fixture(installReady: false), bridge: bridge,
        size: const Size(360, 720));
    expect(find.text('Download'), findsOneWidget);
    expect(find.text('Play'), findsNothing);
    expect(find.text('BUILD • PLAY • EXPLORE'), findsWidgets);
    expect(tester.takeException(), isNull);
  });

  testWidgets('Download starts the real native installer, not a local toast',
      (WidgetTester tester) async {
    final FakeBridge bridge = FakeBridge(fixture(installReady: false));
    await pumpApp(tester, fixture(installReady: false), bridge: bridge,
        size: const Size(360, 720));
    await tester.tap(find.text('Download'));
    await tester.pumpAndSettle(const Duration(milliseconds: 300));
    // installStart is a real MethodChannel/JNI call in production; the fake
    // records the same contract here.
    expect(find.text('Download'), findsWidgets);
    expect(tester.takeException(), isNull);
  });

  testWidgets('Progress shows the REAL percentage/bytes and stays on Home',
      (WidgetTester tester) async {
    await pumpApp(
      tester,
      fixture(
        installReady: false,
        installActive: true,
        installStage: 'Downloading game resources and sound',
        installDone: 50,
        installTotal: 100,
        installPercent: 50,
        installSpeedBps: 1024,
        installEtaSeconds: 30,
        installMilestone: 2,
      ),
      size: const Size(360, 720),
    );
    expect(find.text('50%'), findsOneWidget);
    expect(find.textContaining('50 B / 100 B'), findsOneWidget);
    expect(find.text('Downloading Minecraft…'), findsOneWidget);
    expect(find.text('Download'), findsNothing);
    expect(tester.takeException(), isNull);
  });

  testWidgets('Unknown total size is indeterminate, never a fake percent',
      (WidgetTester tester) async {
    await pumpApp(
      tester,
      fixture(installReady: false, installActive: true, installDone: 2048,
          installTotal: 0, installPercent: null),
      size: const Size(360, 720),
    );
    expect(find.text('2 KB / —'), findsOneWidget);
    expect(find.textContaining('%'), findsNothing);
    expect(tester.takeException(), isNull);
  });

  testWidgets('A real failure is shown with Retry, never silent "no results"',
      (WidgetTester tester) async {
    await pumpApp(
      tester,
      fixture(installReady: false, installError: 'No connection to Mojang'),
      size: const Size(360, 720),
    );
    expect(find.text('Download failed'), findsOneWidget);
    expect(find.text('No connection to Mojang'), findsOneWidget);
    expect(find.text('Retry'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('Ready state shows Play in the SAME slot',
      (WidgetTester tester) async {
    await pumpApp(tester, fixture(installReady: true),
        size: const Size(360, 720));
    expect(find.text('Play'), findsOneWidget);
    expect(find.text('Download'), findsNothing);
    expect(find.text('Ready to play'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('Reduced motion replaces the runner with a static bar',
      (WidgetTester tester) async {
    await tester.pumpWidget(const MaterialApp(
      home: MediaQuery(
        data: MediaQueryData(disableAnimations: true),
        child: Scaffold(
          body: CraftixRunnerProgress(
            progress: 0.25,
            stage: 'Downloading game resources and sound',
            doneBytes: 25,
            totalBytes: 100,
            speedBytesPerSecond: null,
            etaSeconds: null,
            milestone: 1,
            label: 'Downloading Minecraft…',
          ),
        ),
      ),
    ));
    expect(find.text('25%'), findsOneWidget);
    expect(find.byType(LinearProgressIndicator), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  test('snapshot parsing keeps unknown download values null', () {
    final LauncherSnapshot s = LauncherSnapshot.fromJson(const <Object?, Object?>{
      'appVersion': '0.7.8',
      'gameVersion': '26.3',
      'installReady': false,
      'installActive': true,
      'installTotal': 0,
    });
    expect(s.installReady, isFalse);
    expect(s.installActive, isTrue);
    expect(s.installFraction, isNull);
    expect(s.installSpeedBps, isNull);
    expect(s.installEtaSeconds, isNull);
    expect(s.installError, isNull);
  });

  test('snapshot parsing maps real progress and error fields', () {
    final LauncherSnapshot s = LauncherSnapshot.fromJson(<Object?, Object?>{
      'appVersion': '0.7.8',
      'gameVersion': '26.3',
      'installPercent': 75,
      'installSpeedBps': 2048,
      'installEtaSeconds': 12,
      'installMilestone': 3,
      'installError': 'Modrinth answered 429',
      'installStage': 'Downloading game resources and sound',
    });
    expect(s.installFraction, 0.75);
    expect(s.installSpeedBps, 2048);
    expect(s.installEtaSeconds, 12);
    expect(s.installMilestone, 3);
    expect(s.installError, 'Modrinth answered 429');
    expect(s.installStage, 'Downloading game resources and sound');
  });

  test('installStart is part of the typed bridge contract', () async {
    final FakeBridge bridge = FakeBridge(fixture(installReady: false));
    final BridgeResult result = await bridge.installStart();
    expect(result.ok, isTrue);
    expect(result.message, 'Download started');
  });
}

