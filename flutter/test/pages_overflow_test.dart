import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:horizon_launcher/core/widgets/horizon_bottom_nav.dart';

import 'helpers.dart';

const List<Size> targetSizes = <Size>[
  Size(360, 720),
  Size(360, 800),
  Size(393, 873),
  Size(412, 915),
  Size(480, 960),
  Size(800, 1280), // tablet portrait
  Size(960, 540), // landscape fallback
];

void main() {
  for (final Size size in targetSizes) {
    testWidgets('no RenderFlex overflow on any page at ${size.width.toInt()}x${size.height.toInt()}',
        (WidgetTester tester) async {
      await pumpApp(tester, fixture(), size: size);
      for (final String label in <String>['Home', 'Mods', 'Packs', 'Installed', 'Settings']) {
        final Finder item = find.descendant(
          of: find.byType(HorizonBottomNav),
          matching: find.text(label),
        );
        expect(item, findsOneWidget, reason: '$label nav item missing at $size');
        await tester.tap(item);
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 220));
        expect(tester.takeException(), isNull, reason: '$label overflowed at $size');
      }
    });
  }

  testWidgets('bottom nav respects the Android bottom safe area', (WidgetTester tester) async {
    await pumpApp(tester, fixture());
    final RenderBox nav = tester.renderObject<RenderBox>(find.byType(HorizonBottomNav));
    expect(nav.size.width, greaterThan(380));
    expect(nav.size.height, greaterThan(48));
  });

  testWidgets('accounts sheet and value chooser open without overflow', (WidgetTester tester) async {
    await pumpApp(tester, fixture(), size: const Size(360, 720));
    await tester.tap(find.text('Player').first);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 220));
    expect(tester.takeException(), isNull);
    expect(find.text('Sign in with browser'), findsOneWidget);
    await tester.tap(find.text('Sign in with browser'));
    await tester.pump();
    expect(tester.takeException(), isNull);
  });
}
