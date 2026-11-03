import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/bridge/actions.dart';
import '../../core/bridge/models.dart';
import '../../core/state/providers.dart';
import '../../core/theme/horizon_strings.dart';
import '../../core/theme/horizon_theme.dart';
import '../../core/widgets/horizon_widgets.dart';

void showAccountsSheet(BuildContext context, WidgetRef ref, LauncherSnapshot d) {
  showModalBottomSheet<void>(
    context: context,
    isScrollControlled: true,
    useSafeArea: true,
    showDragHandle: true,
    builder: (BuildContext ctx) => _AccountsSheet(d: d, ref: ref),
  );
}

class _AccountsSheet extends ConsumerWidget {
  const _AccountsSheet({required this.d, required this.ref});
  final LauncherSnapshot d;
  final WidgetRef ref;

  @override
  Widget build(BuildContext context, WidgetRef _) {
    final HorizonStrings s = ref.watch(stringsProvider);
    return Padding(
      padding: EdgeInsets.only(
        left: 18, right: 18, top: 8, bottom: MediaQuery.of(context).viewInsets.bottom + 20),
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            SectionTitle(s.t('accounts'), subtitle: d.accountStatus ?? s.t('loginStatus')),
            if (d.avatarPath != null)
              Padding(
                padding: const EdgeInsets.only(bottom: 12),
                child: CircleAvatar(radius: 30, backgroundImage: AssetImage(d.avatarPath!)),
              ),
            HorizonCard(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: <Widget>[
                  Text(s.t('loginStatus'), style: const TextStyle(fontSize: 11, color: HorizonPalette.textSecondary)),
                  const SizedBox(height: 4),
                  Text(d.accountStatus ?? d.profileName ?? '—',
                      style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w700)),
                ],
              ),
            ),
            const SizedBox(height: 12),
            GradientButton(label: s.t('browserLogin'), icon: Icons.public_rounded,
                onTap: () => ActionRunner(ref, context).run(HorizonAction.SignIn)),
            const SizedBox(height: 10),
            GradientButton(label: s.t('deviceCode'), icon: Icons.pin_rounded,
                onTap: () => ActionRunner(ref, context).run(HorizonAction.DeviceSignIn)),
            const SizedBox(height: 10),
            _OutlineButton(s.t('offlineName'), Icons.person_outline_rounded,
                () => ActionRunner(ref, context).run(HorizonAction.Field, args: <String, Object?>{'name': 'name'})),
            const SizedBox(height: 10),
            _OutlineButton(s.t('signOut'), Icons.logout_rounded,
                () => ActionRunner(ref, context).run(HorizonAction.SignOut)),
            const SizedBox(height: 8),
            Text(
              'Microsoft passwords are never stored by Horizon; browser/device-code '
              'authentication and the Android Keystore stay native.',
              style: TextStyle(fontSize: 11, color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.6)),
            ),
          ],
        ),
      ),
    );
  }
}

void showChooserSheet(BuildContext context, WidgetRef ref, HorizonAction action) {
  showModalBottomSheet<void>(
    context: context,
    useSafeArea: true,
    showDragHandle: true,
    builder: (BuildContext ctx) => _ChooserSheet(action: action, ref: ref),
  );
}

class _ChooserSheet extends ConsumerWidget {
  const _ChooserSheet({required this.action, required this.ref});
  final HorizonAction action;
  final WidgetRef ref;

  @override
  Widget build(BuildContext context, WidgetRef _) {
    final HorizonStrings s = ref.watch(stringsProvider);
    final LauncherSnapshot? d = ref.watch(snapshotProvider).valueOrNull;
    // Options come from the native host's real enum values, never a Dart list.
    final List<String> options = d?.choices[action.name] ?? const <String>[];
    return Padding(
      padding: const EdgeInsets.fromLTRB(18, 8, 18, 24),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          SectionTitle(action.name),
          if (options.isEmpty)
            Padding(
              padding: const EdgeInsets.all(16),
              child: Text(s.t('noResultsHint'), textAlign: TextAlign.center,
                  style: const TextStyle(fontSize: 12, color: HorizonPalette.textSecondary)),
            )
          else
            ...options.map((String option) => Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: HorizonCard(
                    onTap: () {
                      Navigator.of(context).pop();
                      ActionRunner(ref, context).run(action, args: <String, Object?>{'value': option});
                    },
                    padding: const EdgeInsets.all(14),
                    child: Text(option, style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600)),
                  ),
                )),
        ],
      ),
    );
  }
}

void showImportDialog(BuildContext context, WidgetRef ref, ContentKind kind) {
  showModalBottomSheet<void>(
    context: context,
    useSafeArea: true,
    showDragHandle: true,
    builder: (BuildContext ctx) => Padding(
      padding: const EdgeInsets.fromLTRB(18, 8, 18, 24),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          SectionTitle('Import ${kind.name}'),
          Text(
            'Files are picked with the Android Storage Access Framework and '
            'validated by the native host before they enter the content folder.',
            style: const TextStyle(fontSize: 12, color: HorizonPalette.textSecondary),
          ),
          const SizedBox(height: 16),
          GradientButton(
            label: 'Choose file',
            icon: Icons.file_open_rounded,
            onTap: () {
              Navigator.of(ctx).pop();
              ActionRunner(ref, context).run(
                HorizonAction.Import,
                args: <String, Object?>{'kind': kind.name},
              );
            },
          ),
        ],
      ),
    ),
  );
}

class _OutlineButton extends StatelessWidget {
  const _OutlineButton(this.label, this.icon, this.onTap);
  final String label;
  final IconData icon;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return HorizonPressable(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(vertical: 13),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(HorizonRadii.control),
          border: Border.all(color: HorizonPalette.border),
          color: HorizonPalette.card,
        ),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            Icon(icon, size: 18, color: HorizonPalette.textSecondary),
            const SizedBox(width: 8),
            Flexible(child: Text(label, overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 14))),
          ],
        ),
      ),
    );
  }
}
