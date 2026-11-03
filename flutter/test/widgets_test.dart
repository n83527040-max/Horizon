import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:horizon_launcher/core/widgets/horizon_bottom_nav.dart';
import 'package:horizon_launcher/core/widgets/horizon_widgets.dart';
import 'package:horizon_launcher/features/home/home_page.dart';

import 'helpers.dart';

void main() {
  testWidgets('bottom navigation has 5 real destinations and fills the width', (WidgetTester tester) async {
    await pumpApp(tester, fixture());
    expect(find.byType(HorizonBottomNav), findsOneWidget);
    for (final String label in <String>['Home', 'Mods', 'Packs', 'Installed', 'Settings']) {
      expect(find.text(label), findsWidgets, reason: '$label missing');
    }
    final RenderBox navBox = tester.renderObject<RenderBox>(find.byType(HorizonBottomNav));
    expect(navBox.size.width, closeTo(393, 0.5));
  });

  testWidgets('pressing a tab keeps other pages alive (IndexedStack)', (WidgetTester tester) async {
    await pumpApp(tester, fixture());
    await tester.tap(find.text('Mods').last);
    await tester.pump();
    expect(find.byType(IndexedStack), findsOneWidget);
    // Home is still mounted (state preserved), just offstage.
    expect(find.byType(HomePage, skipOffstage: false), findsWidgets);
  });

  testWidgets('Hero, Play button and real performance values render', (WidgetTester tester) async {
    await pumpApp(tester, fixture());
    expect(find.text('Play'), findsOneWidget);
    expect(find.text('HORIZON'), findsWidgets);
    expect(find.text('267 MB / 3.7 GB'), findsOneWidget);
    expect(find.text('Vulkan / compatibility'), findsWidgets);
  });

  testWidgets('unavailable real values are hidden, not replaced by demo numbers', (WidgetTester tester) async {
    await pumpApp(tester, fixture(rich: false));
    expect(find.text('Vulkan / compatibility'), findsNothing);
    expect(find.text('267 MB / 3.7 GB'), findsNothing);
    expect(find.text('No content installed'), findsOneWidget);
  });
}
