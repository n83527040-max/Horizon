import 'package:flutter_test/flutter_test.dart';
import 'package:horizon_launcher/core/theme/horizon_motion.dart';
import 'package:horizon_launcher/core/theme/horizon_strings.dart';

import 'helpers.dart';

void main() {
  test('Latin/numeric fragments get an explicit LTR embedding for RTL Arabic', () {
    final String wrapped = ltr('267 MB / 3.7 GB');
    expect(wrapped.startsWith('\u202A'), isTrue);
    expect(wrapped.endsWith('\u202C'), isTrue);
    expect(wrapped.contains('267 MB / 3.7 GB'), isTrue);
  });

  test('every new UI key has an Arabic translation', () {
    const HorizonStrings s = HorizonStrings('ar');
    for (final String key in <String>[
      'networkError', 'apiError', 'parseError', 'retry', 'addOffline',
      'offlineInvalid', 'microsoftAccounts', 'offlineAccounts', 'projectDetails',
      'resourcePacks', 'installedSubtitle', 'modsSubtitle', 'packsSubtitle',
      'shadersSubtitle', 'footerNote', 'loadingContent',
    ]) {
      expect(s.t(key), isNot(key), reason: '$key is missing its Arabic text');
      expect(s.t(key).isNotEmpty, isTrue);
    }
  });

  test('real native messages are localized but unknown text is never invented', () {
    const HorizonStrings s = HorizonStrings('ar');
    expect(s.native('Catalog source saved'), 'تم حفظ مصدر المحتوى');
    expect(s.native('Unmapped native status 42'), 'Unmapped native status 42');
  });

  test('fixtures with an error are classified, not hidden', () {
    final snapshot = fixture(errorKind: 'api', catalogStatus: 'Modrinth answered 429.');
    expect(snapshot.hasError, isTrue);
    expect(snapshot.canRetry, isTrue);
    expect(snapshot.catalogStatus, 'Modrinth answered 429.');
  });
}
