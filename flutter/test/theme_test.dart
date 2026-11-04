import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:horizon_launcher/core/theme/horizon_strings.dart';
import 'package:horizon_launcher/core/theme/horizon_theme.dart';

void main() {
  test('palette matches the supplied reference identity', () {
    expect(HorizonPalette.background, const Color(0xFF070B1D));
    expect(HorizonPalette.primary, const Color(0xFF8B35F5));
    expect(HorizonPalette.secondary, const Color(0xFF315BFF));
    expect(HorizonPalette.accent, const Color(0xFF22D3EE));
    expect(HorizonPalette.card, const Color(0xFF111B38));
  });

  test('dark and light themes are both Material 3 and usable', () {
    final ThemeData dark = HorizonTheme.dark();
    final ThemeData light = HorizonTheme.light();
    expect(dark.useMaterial3, isTrue);
    expect(light.useMaterial3, isTrue);
    expect(dark.brightness, Brightness.dark);
    expect(light.brightness, Brightness.light);
    expect(dark.scaffoldBackgroundColor, HorizonPalette.background);
    expect(dark.colorScheme.primary, HorizonPalette.primary);
  });

  test('responsive metrics never create a narrow desktop column', () {
    for (final Size size in <Size>[
      const Size(360, 720),
      const Size(360, 800),
      const Size(393, 873),
      const Size(412, 915),
      const Size(480, 960),
      const Size(800, 1280),
      const Size(960, 540),
    ]) {
      final HorizonMetrics m = HorizonMetrics(size, const EdgeInsets.only(top: 24, bottom: 16));
      expect(m.horizontal, lessThanOrEqualTo(32));
      expect(m.horizontal, greaterThanOrEqualTo(16));
      // At 393dp the content keeps >90% of the screen width.
      if (size.width == 393) {
        expect(size.width - 2 * m.horizontal, greaterThan(size.width * 0.9));
      }
      expect(m.heroHeight, inInclusiveRange(150.0, 260.0));
      expect(m.fontScale, inInclusiveRange(0.88, 1.18));
    }
  });

  test('CRAFTIX identity is applied in both languages', () {
    expect(HorizonStrings.of('en').t('appName'), 'CRAFTIX');
    expect(HorizonStrings.of('ar').t('appName'), 'CRAFTIX');
    expect(HorizonStrings.of('en').t('appSubtitle'), 'Minecraft Java • Android');
    expect(HorizonStrings.of('en').t('tagline'), 'BUILD • PLAY • EXPLORE');
  });

  test('language is a real applied setting', () {
    expect(HorizonStrings.of('ar').t('home'), 'الرئيسية');
    expect(HorizonStrings.of('en').t('home'), 'Home');
    expect(HorizonStrings.of(null).t('home'), 'Home');
    expect(HorizonStrings.supported, contains(const Locale('ar')));
  });
}
