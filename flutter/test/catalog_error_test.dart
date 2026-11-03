import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:horizon_launcher/core/bridge/models.dart';
import 'package:horizon_launcher/core/widgets/horizon_bottom_nav.dart';

import 'helpers.dart';

/// A real network/API/parse failure must NEVER be shown as "No results".
void main() {
  const List<MapEntry<String, String>> cases = <MapEntry<String, String>>[
    MapEntry<String, String>('network', 'Network error'),
    MapEntry<String, String>('api', 'Source API error'),
    MapEntry<String, String>('parse', 'Invalid API response'),
    MapEntry<String, String>('storage', 'Storage / permission error'),
    MapEntry<String, String>('auth', 'Authentication error'),
  ];
  for (final MapEntry<String, String> entry in cases) {
    final String kind = entry.key;
    final String title = entry.value;
    testWidgets('$kind error shows its real reason and a retry', (WidgetTester tester) async {
      final FakeBridge bridge = FakeBridge(fixture(
        errorKind: kind,
        catalogStatus: 'real native failure for $kind',
        results: const <CatalogProject>[],
        pending: false,
      ));
      await pumpApp(tester, fixture(
        errorKind: kind,
        catalogStatus: 'real native failure for $kind',
        results: const <CatalogProject>[],
      ), bridge: bridge, size: const Size(393, 873));

      await tester.tap(find.descendant(
          of: find.byType(HorizonBottomNav), matching: find.text('Mods')));
      await tester.pump(const Duration(milliseconds: 260));
      await tester.pump();

      expect(find.text(title), findsWidgets, reason: '$kind must show its real title');
      expect(find.text('No results yet'), findsNothing,
          reason: '$kind must not be hidden behind a generic empty state');
      expect(find.text('Retry'), findsWidgets);
      expect(find.textContaining('real native failure'), findsWidgets,
          reason: 'the real native status must be visible');
      expect(tester.takeException(), isNull);
    });
  }

  testWidgets('retry dispatches a real JNI retry operation', (WidgetTester tester) async {
    final FakeBridge bridge = FakeBridge(fixture(
      errorKind: 'network',
      catalogStatus: 'No connection to Modrinth',
      results: const <CatalogProject>[],
    ));
    await pumpApp(tester, fixture(
      errorKind: 'network',
      catalogStatus: 'No connection to Modrinth',
      results: const <CatalogProject>[],
    ), bridge: bridge, size: const Size(393, 873));
    await tester.tap(find.descendant(
        of: find.byType(HorizonBottomNav), matching: find.text('Mods')));
    await tester.pump(const Duration(milliseconds: 260));
    await tester.pump();
    await tester.tap(find.text('Retry').first);
    await tester.pump();
    expect(
      bridge.dispatches.any((Map<String, Object?> d) =>
          d['action'] == 'Field' &&
          (d['args'] as Map<String, Object?>)['name'] == 'retry'),
      isTrue,
    );
  });

  testWidgets('real loading state shows a skeleton, not fake results', (WidgetTester tester) async {
    await pumpApp(tester, fixture(
      pending: true,
      results: const <CatalogProject>[],
      progress: 0.4,
    ), size: const Size(393, 873));
    await tester.tap(find.descendant(
        of: find.byType(HorizonBottomNav), matching: find.text('Mods')));
    await tester.pump(const Duration(milliseconds: 260));
    await tester.pump();
    expect(find.text('Loading real content…'), findsWidgets);
    expect(find.byType(LinearProgressIndicator), findsWidgets);
    expect(tester.takeException(), isNull);
  });
}
