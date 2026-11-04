import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:horizon_launcher/core/bridge/actions.dart';

/// Cross-language contract: the Dart UI must not invent or drop an action.
/// This test parses the actual Rust source, so a change to the native action
/// set fails the Flutter test suite until the bridge is updated.
List<String> rustVariants(String path, String enumName) {
  final String source = File(path).readAsStringSync();
  final RegExpMatch? body = RegExp(
    r'enum ' + enumName + r' \{([\s\S]*?)\n\}',
  ).firstMatch(source);
  expect(body, isNotNull, reason: '$enumName not found in $path');
  return body!
      .group(1)!
      .split('\n')
      .map((String line) => line.trim())
      .where((String line) => line.isNotEmpty && !line.startsWith('//') && !line.startsWith('#'))
      .map((String line) => RegExp(r'^([A-Z][A-Za-z0-9_]*)').firstMatch(line)?.group(1))
      .whereType<String>()
      .toList();
}

void main() {
  const String launcherPages = '../crates/horizon-client/src/android/launcher_pages.rs';
  const String catalog = '../crates/horizon-client/src/android/catalog.rs';
  const String contentFiles = '../crates/horizon-client/src/content_files.rs';

  test('every Rust Action has a typed Dart counterpart', () {
    final List<String> rust = rustVariants(launcherPages, 'Action');
    final List<String> dart = HorizonAction.values.map((HorizonAction a) => a.name).toList();
    expect(rust.length, 49, reason: 'Rust Action enum changed');
    expect(dart.toSet(), equals(rust.toSet()));
    expect(dart.length, dart.toSet().length, reason: 'duplicate Dart actions');
  });

  test('page and content-kind enums match the native definitions', () {
    expect(
      HorizonPage.values.map((HorizonPage p) => p.name).toSet(),
      equals(rustVariants(catalog, 'Page').toSet()),
    );
    expect(
      ContentKind.values.map((ContentKind k) => k.name).toSet(),
      equals(rustVariants(contentFiles, 'Kind').toSet()),
    );
  });

  test('content folder names are the real engine folders', () {
    expect(ContentKind.Mods.folder, 'horizon-mods');
    expect(ContentKind.Packs.folder, 'resourcepacks');
    expect(ContentKind.Shaders.folder, 'shaderpacks');
  });
}
