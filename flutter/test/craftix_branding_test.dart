import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:horizon_launcher/core/theme/horizon_strings.dart';
import 'package:horizon_launcher/core/theme/horizon_theme.dart';

void main() {
  test('CRAFTIX identity is applied in both languages', () {
    for (final String code in <String>['en', 'ar']) {
      final HorizonStrings s = HorizonStrings.of(code);
      expect(s.t('appName'), 'CRAFTIX');
      expect(s.t('appSubtitle'), 'Minecraft Java • Android');
      expect(s.t('tagline').isNotEmpty, isTrue);
    }
  });

  test('CRAFTIX palette matches the supplied reference', () {
    expect(HorizonPalette.background, const Color(0xFF070B1D));
    expect(HorizonPalette.card, const Color(0xFF111B38));
    expect(HorizonPalette.primary, const Color(0xFF8B35F5));
    expect(HorizonPalette.secondary, const Color(0xFF315BFF));
    expect(HorizonPalette.accent, const Color(0xFF22D3EE));
    expect(HorizonPalette.textPrimary, const Color(0xFFF7F8FF));
    expect(HorizonPalette.textSecondary, const Color(0xFFA8B1CC));
  });

  test('user-facing Dart source contains no HORIZON branding', () {
    final Directory lib = Directory('lib');
    final List<File> dart = lib
        .listSync(recursive: true)
        .whereType<File>()
        .where((File f) => f.path.endsWith('.dart'))
        .toList();
    expect(dart.length, greaterThan(10));
    for (final File file in dart) {
      expect(file.readAsStringSync().contains('HORIZON'), isFalse,
          reason: 'HORIZON branding remains in ${file.path}');
    }
  });

  test('Android identity is CRAFTIX / com.craftix.launcher', () {
    final String flutterManifest =
        File('../flutter/android/app/src/main/AndroidManifest.xml').readAsStringSync();
    expect(flutterManifest, contains('android:label="CRAFTIX"'));
    expect(flutterManifest, contains('@mipmap/ic_launcher'));
    final String gradle = File('../packaging/android/build.gradle').readAsStringSync();
    expect(gradle, contains("namespace 'com.craftix.launcher'"));
    expect(gradle, contains("versionName '0.7.8'"));
    expect(gradle, contains('versionCode 30'));
  });
}
