import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:horizon_launcher/core/bridge/models.dart';

import 'helpers.dart';

/// REAL account paths: offline creation/validation/selection/deletion and
/// Microsoft error display. These are widget/contract tests; live Microsoft
/// OAuth against microsoftonline.com is NOT claimed here.
void main() {
  testWidgets('offline account creation opens a real input and rejects invalid names',
      (WidgetTester tester) async {
    final FakeBridge bridge = FakeBridge(fixture());
    await pumpApp(tester, fixture(), bridge: bridge, size: const Size(393, 873));
    await tester.tap(find.text('Player').first);
    await tester.pump(const Duration(milliseconds: 260));
    await tester.pump();

    expect(find.text('Add offline account'), findsOneWidget,
        reason: 'the offline account button must exist');
    await tester.tap(find.text('Add offline account'));
    await tester.pump(const Duration(milliseconds: 260));
    await tester.pump();

    expect(find.byType(TextField), findsOneWidget, reason: 'a real name input is required');
    await tester.enterText(find.byType(TextField), 'ab');
    await tester.tap(find.text('Add').last);
    await tester.pump();
    expect(find.text('Enter a valid player name (3–16 letters, digits or _)'), findsOneWidget,
        reason: 'an invalid name must show a real error, not a fake success');
    expect(bridge.dispatches, isEmpty, reason: 'no native call for an invalid name');

    await tester.enterText(find.byType(TextField), 'Steve_123');
    await tester.tap(find.text('Add').last);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 200));
    expect(
      bridge.dispatches.any((Map<String, Object?> d) =>
          d['action'] == 'Field' &&
          (d['args'] as Map<String, Object?>)['name'] == 'offline/add' &&
          (d['args'] as Map<String, Object?>)['value'] == 'Steve_123'),
      isTrue,
      reason: 'a valid name must reach the real JNI account operation',
    );
  });

  testWidgets('real account list offers select and delete for each account',
      (WidgetTester tester) async {
    await pumpApp(
      tester,
      fixture(accounts: const <LauncherAccount>[
        LauncherAccount(id: 'u1', name: 'MSPlayer', type: 'microsoft', selected: true),
        LauncherAccount(id: 'Steve', name: 'Steve', type: 'offline'),
      ]),
      size: const Size(393, 873),
    );
    await tester.tap(find.text('Player').first);
    await tester.pump(const Duration(milliseconds: 260));
    await tester.pump();
    expect(find.text('Microsoft accounts'), findsOneWidget);
    expect(find.text('Offline accounts'), findsOneWidget);
    expect(find.text('MSPlayer'), findsWidgets);
    expect(find.text('Steve'), findsWidgets);
    expect(find.byTooltip('Select'), findsNWidgets(2));
    expect(find.byTooltip('Delete'), findsNWidgets(2));
    expect(tester.takeException(), isNull);
  });

  testWidgets('Microsoft authentication errors show the real stage and a retry',
      (WidgetTester tester) async {
    await pumpApp(
      tester,
      fixture(catalogStatus: 'XSTS returned 401').copyWith(
        loginMessage: 'XSTS returned 401: token expired',
        authStage: 'xsts',
        errorKind: 'auth',
      ),
      size: const Size(393, 873),
    );
    await tester.tap(find.text('Player').first);
    await tester.pump(const Duration(milliseconds: 260));
    await tester.pump();
    expect(find.text('Authentication error'), findsWidgets);
    expect(find.text('\u202Axsts\u202C'), findsWidgets);
    expect(find.text('Retry'), findsWidgets);
    expect(tester.takeException(), isNull);
  });
}

extension _SnapshotCopy on LauncherSnapshot {
  LauncherSnapshot copyWith({
    String? loginMessage,
    String? authStage,
    String? errorKind,
  }) =>
      LauncherSnapshot(
        appVersion: appVersion,
        gameVersion: gameVersion,
        profileName: profileName,
        accountStatus: accountStatus,
        avatarPath: avatarPath,
        graphicsApi: graphicsApi,
        renderScalePercent: renderScalePercent,
        ramUsedMb: ramUsedMb,
        ramPeakMb: ramPeakMb,
        ramTotalMb: ramTotalMb,
        installed: installed,
        results: results,
        status: status,
        settings: settings,
        choices: choices,
        storageBytes: storageBytes,
        cacheBytes: cacheBytes,
        contentBytes: contentBytes,
        engineBridgeReady: engineBridgeReady,
        pending: pending,
        loginCode: loginCode,
        loginUri: loginUri,
        loginMessage: loginMessage ?? this.loginMessage,
        catalogStatus: catalogStatus,
        errorKind: errorKind ?? this.errorKind,
        canRetry: canRetry,
        authStage: authStage ?? this.authStage,
        progress: progress,
        activePage: activePage,
        filterVersion: filterVersion,
        filterLoader: filterLoader,
        accounts: accounts,
      );
}
