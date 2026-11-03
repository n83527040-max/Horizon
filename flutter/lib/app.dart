import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'core/state/providers.dart';
import 'core/theme/horizon_strings.dart';
import 'core/theme/horizon_theme.dart';
import 'navigation/app_shell.dart';

class HorizonApp extends ConsumerWidget {
  const HorizonApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final ThemeMode mode = ref.watch(themeModeProvider);
    final Locale locale = ref.watch(localeProvider);
    return MaterialApp(
      title: 'CRAFTIX',
      debugShowCheckedModeBanner: false,
      theme: HorizonTheme.light(),
      darkTheme: HorizonTheme.dark(),
      themeMode: mode == ThemeMode.system ? ThemeMode.dark : mode,
      locale: locale,
      supportedLocales: HorizonStrings.supported,
      localizationsDelegates: GlobalMaterialLocalizations.delegates,
      home: const AppShell(),
    );
  }
}
