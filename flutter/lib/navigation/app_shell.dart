import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../core/bridge/actions.dart';
import '../core/state/providers.dart';
import '../core/theme/horizon_strings.dart';
import '../core/theme/horizon_motion.dart';
import '../core/theme/horizon_theme.dart';
import '../core/widgets/horizon_bottom_nav.dart';
import '../features/catalog/catalog_page.dart';
import '../features/home/home_page.dart';
import '../features/installed/installed_page.dart';
import '../features/settings/settings_page.dart';

/// IndexedStack keeps every page alive: switching tabs never reloads native
/// data or loses scroll position. Android Back closes an open sheet first,
/// then returns to Home, then exits.
class AppShell extends ConsumerStatefulWidget {
  const AppShell({super.key});

  @override
  ConsumerState<AppShell> createState() => _AppShellState();
}

class _AppShellState extends ConsumerState<AppShell> with WidgetsBindingObserver {
  static const List<Widget> _pages = <Widget>[
    HomePage(),
    CatalogPage(kind: ContentKind.Mods),
    CatalogPage(kind: ContentKind.Packs),
    InstalledPage(),
    SettingsPage(),
    CatalogPage(kind: ContentKind.Shaders),
  ];

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    // A catalog page must tell Rust its real Page before browse() works.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _autoBrowse(ref.read(navIndexProvider));
    });
  }

  void _autoBrowse(int index) {
    final String? kind = switch (index) {
      1 => 'Mods',
      2 => 'Packs',
      5 => 'Shaders',
      _ => null,
    };
    if (kind == null) return;
    // Direct dispatch: no SnackBar for an automatic background browse.
    ref.read(bridgeProvider).dispatch(HorizonAction.Page,
        args: <String, Object?>{'kind': kind});
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  Future<bool> didPopRoute() async {
    final NavigatorState navigator = Navigator.of(context);
    if (navigator.canPop()) {
      await navigator.maybePop();
      return true;
    }
    final int index = ref.read(navIndexProvider);
    if (index != 0) {
      ref.read(navIndexProvider.notifier).state = 0;
      return true;
    }
    return false;
  }

  @override
  Widget build(BuildContext context) {
    final int index = ref.watch(navIndexProvider);
    ref.listen<int>(navIndexProvider, (int? previous, int next) {
      if (previous != next) _autoBrowse(next);
    });
    final HorizonStrings s = ref.watch(stringsProvider);
    // Asynchronous real results (SAF import, game hand-off failure, sign-in
    // code) arrive after the method call returned. Show their real outcome.
    ref.listen<AsyncValue<Map<String, Object?>>>(nativeEventsProvider,
        (AsyncValue<Map<String, Object?>>? previous, AsyncValue<Map<String, Object?>> next) {
      next.whenData((Map<String, Object?> value) {
        final bool ok = value['ok'] == true;
        final String message = value['message'] is String
            ? value['message']! as String
            : 'Native operation finished';
        ScaffoldMessenger.of(context)
          ..hideCurrentSnackBar()
          ..showSnackBar(SnackBar(
            content: Text(s.native(message)),
            behavior: SnackBarBehavior.floating,
            margin: const EdgeInsets.fromLTRB(16, 0, 16, 82),
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(HorizonRadii.control)),
            backgroundColor: ok ? HorizonPalette.card : HorizonPalette.danger,
          ));
        ref.invalidate(snapshotProvider);
      });
    });
    final snapshot = ref.watch(snapshotProvider);
    final bool pending = snapshot.valueOrNull?.pending ?? false;
    final List<String> labels = <String>[
      s.t('home'), s.t('mods'), s.t('packs'), s.t('installed'), s.t('settings'),
    ];
    return Scaffold(
      // The keyboard/IME must never resize the shell or shift the bottom nav.
      resizeToAvoidBottomInset: false,
      body: SafeArea(
        top: false,
        bottom: false,
        child: Column(
          children: <Widget>[
            if (pending) const LinearProgressIndicator(minHeight: 2),
            Expanded(
              // A short animated transition WITHOUT replacing children:
              // IndexedStack still preserves every page's state and scroll.
              child: AnimatedPageFade(
                index: index,
                child: IndexedStack(
                  index: index.clamp(0, _pages.length - 1),
                  children: _pages,
                ),
              ),
            ),
            HorizonBottomNav(
              labels: labels,
              index: index > 4 ? 1 : index,
              onSelected: (int i) => ref.read(navIndexProvider.notifier).state = i,
            ),
          ],
        ),
      ),
    );
  }
}

/// Re-export for pages that need to open the Shaders page from Home/Manage.
void openShaders(WidgetRef ref) => ref.read(navIndexProvider.notifier).state = 5;

/// Horizontally adaptive content width. Phones always get full width; only
/// wide tablets get a modest margin (never a narrow desktop column).
double contentWidth(HorizonMetrics m) => m.horizontal;
