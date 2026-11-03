import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/bridge/actions.dart';
import '../../core/bridge/models.dart';
import '../../core/state/providers.dart';
import '../../core/theme/horizon_motion.dart';
import '../../core/theme/horizon_strings.dart';
import '../../core/theme/horizon_theme.dart';
import '../../core/widgets/horizon_widgets.dart';

void showAccountsSheet(BuildContext context, WidgetRef ref, LauncherSnapshot d) {
  showModalBottomSheet<void>(
    context: context,
    isScrollControlled: true,
    useSafeArea: true,
    showDragHandle: true,
    backgroundColor: Colors.transparent,
    builder: (BuildContext ctx) => AnimatedSheet(child: _AccountsSheet(d: d, ref: ref)),
  );
}

/// REAL accounts: Microsoft (Accounts::accounts) + offline (Accounts::offline).
/// Every button performs a real JNI operation; no dummy success.
class _AccountsSheet extends ConsumerWidget {
  const _AccountsSheet({required this.d, required this.ref});
  final LauncherSnapshot d;
  final WidgetRef ref;

  @override
  Widget build(BuildContext context, WidgetRef _) {
    final HorizonStrings s = ref.watch(stringsProvider);
    final LauncherSnapshot current = ref.watch(snapshotProvider).valueOrNull ?? d;
    final List<LauncherAccount> microsoft = current.accounts
        .where((LauncherAccount a) => !a.isOffline)
        .toList(growable: false);
    final List<LauncherAccount> offline = current.accounts
        .where((LauncherAccount a) => a.isOffline)
        .toList(growable: false);
    return Container(
      decoration: BoxDecoration(
        color: HorizonPalette.background,
        borderRadius: const BorderRadius.vertical(top: Radius.circular(HorizonRadii.card)),
      ),
      padding: EdgeInsets.only(
        left: 18, right: 18, top: 8,
        bottom: MediaQuery.of(context).viewInsets.bottom + 20,
      ),
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            SectionTitle(s.t('accounts'), subtitle: current.accountStatus ?? s.t('loginStatus')),
            if (current.avatarPath != null)
              Padding(
                padding: const EdgeInsets.only(bottom: 12),
                child: CircleAvatar(radius: 30, backgroundImage: AssetImage(current.avatarPath!)),
              ),
            HorizonCard(
              child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: <Widget>[
                Text(s.t('loginStatus'), style: const TextStyle(fontSize: 11, color: HorizonPalette.textSecondary)),
                const SizedBox(height: 4),
                Text(
                  ltr(current.accountStatus ?? current.profileName ?? '—'),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w700),
                ),
              ]),
            ),
            // Real Microsoft sign-in state / device code / error with stage.
            if (current.loginCode != null || current.loginMessage != null) ...<Widget>[
              const SizedBox(height: 12),
              _SignInStateCard(s: s, current: current, ref: ref, context: context),
            ],
            const SizedBox(height: 14),
            _AccountGroup(
              title: s.t('microsoftAccounts'),
              accounts: microsoft,
              empty: s.t('noAccounts'),
              s: s,
              onSelect: (LauncherAccount a) => ActionRunner(ref, context).run(HorizonAction.Field,
                  args: <String, Object?>{'name': 'account/select', 'id': a.id, 'value': a.id}),
              onDelete: (LauncherAccount a) => ActionRunner(ref, context).run(HorizonAction.Field,
                  args: <String, Object?>{'name': 'account/delete', 'id': a.id, 'value': a.id}),
            ),
            const SizedBox(height: 10),
            _AccountGroup(
              title: s.t('offlineAccounts'),
              accounts: offline,
              empty: s.t('noAccounts'),
              s: s,
              onSelect: (LauncherAccount a) => ActionRunner(ref, context).run(HorizonAction.Field,
                  args: <String, Object?>{'name': 'offline/select', 'value': a.name}),
              onDelete: (LauncherAccount a) => ActionRunner(ref, context).run(HorizonAction.Field,
                  args: <String, Object?>{'name': 'offline/delete', 'value': a.name}),
            ),
            const SizedBox(height: 12),
            GradientButton(label: s.t('addOffline'), icon: Icons.person_add_alt_rounded,
                onTap: () => _addOffline(context, ref, s)),
            const SizedBox(height: 10),
            GradientButton(label: s.t('browserLogin'), icon: Icons.public_rounded,
                onTap: () => ActionRunner(ref, context).run(HorizonAction.SignIn)),
            const SizedBox(height: 10),
            GradientButton(label: s.t('deviceCode'), icon: Icons.pin_rounded,
                onTap: () => ActionRunner(ref, context).run(HorizonAction.DeviceSignIn)),
            const SizedBox(height: 10),
            _OutlineButton(s.t('refreshSession'), Icons.refresh_rounded,
                () => ActionRunner(ref, context).run(HorizonAction.Field,
                    args: <String, Object?>{'name': 'account/refresh'})),
            const SizedBox(height: 10),
            _OutlineButton(s.t('signOut'), Icons.logout_rounded,
                () => ActionRunner(ref, context).run(HorizonAction.SignOut)),
            const SizedBox(height: 8),
            Text(
              s.arabic
                  ? 'لا تُخزَّن كلمات مرور Microsoft في HORIZON؛ المصادقة وAndroid Keystore تبقى أصلية.'
                  : 'Microsoft passwords are never stored by Horizon; browser/device-code authentication and the Android Keystore stay native.',
              style: TextStyle(fontSize: 11,
                  color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.6)),
            ),
          ],
        ),
      ),
    );
  }

  /// A real text input. Validation matches `Accounts::valid_offline_name`.
  Future<void> _addOffline(BuildContext context, WidgetRef ref, HorizonStrings s) async {
    final TextEditingController controller = TextEditingController();
    String? error;
    final String? name = await showDialog<String>(
      context: context,
      builder: (BuildContext ctx) => StatefulBuilder(
        builder: (BuildContext ctx, StateSetter setState) => AlertDialog(
          backgroundColor: HorizonPalette.card,
          title: Text(s.t('addOffline'), style: const TextStyle(fontSize: 16)),
          content: TextField(
            controller: controller,
            autofocus: true,
            textDirection: TextDirection.ltr,
            maxLength: 16,
            decoration: InputDecoration(
              hintText: s.t('offlineHint'),
              errorText: error,
            ),
            onSubmitted: (String value) {
              final String trimmed = value.trim();
              if (_validName(trimmed)) {
                Navigator.of(ctx).pop(trimmed);
              } else {
                setState(() => error = s.t('offlineInvalid'));
              }
            },
          ),
          actions: <Widget>[
            TextButton(onPressed: () => Navigator.of(ctx).pop(), child: Text(s.t('cancel'))),
            FilledButton(
              onPressed: () {
                final String trimmed = controller.text.trim();
                if (_validName(trimmed)) {
                  Navigator.of(ctx).pop(trimmed);
                } else {
                  setState(() => error = s.t('offlineInvalid'));
                }
              },
              child: Text(s.t('add')),
            ),
          ],
        ),
      ),
    );
    if (name == null) return;
    await ActionRunner(ref, context).run(HorizonAction.Field,
        args: <String, Object?>{'name': 'offline/add', 'value': name});
  }

  static bool _validName(String value) => value.length >= 3 &&
      value.length <= 16 &&
      RegExp(r'^[A-Za-z0-9_]+$').hasMatch(value);
}

class _SignInStateCard extends StatelessWidget {
  const _SignInStateCard({required this.s, required this.current, required this.ref, required this.context});
  final HorizonStrings s;
  final LauncherSnapshot current;
  final WidgetRef ref;
  final BuildContext context;

  @override
  Widget build(BuildContext _) {
    final bool failed = current.loginMessage != null &&
        current.authStage != 'done' &&
        !current.loginMessage!.startsWith('Signed in');
    return HorizonCard(
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: <Widget>[
        Row(children: <Widget>[
          Icon(failed ? Icons.error_outline_rounded : Icons.pin_rounded,
              color: failed ? HorizonPalette.danger : HorizonPalette.accent, size: 18),
          const SizedBox(width: 8),
          Expanded(child: Text(failed ? s.t('authError') : s.t('deviceCode'),
              style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w700))),
          if (current.authStage != null) _StageTag(ltr(current.authStage!)),
        ]),
        const SizedBox(height: 8),
        if (current.loginCode != null) ...<Widget>[
          Text(ltr(current.loginCode!),
              style: const TextStyle(fontSize: 24, fontWeight: FontWeight.w800, letterSpacing: 2)),
          const SizedBox(height: 4),
          Text(s.t('deviceCodeHint'), style: const TextStyle(fontSize: 11)),
        ],
        if (current.loginUri != null)
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Text(ltr(current.loginUri!), style: const TextStyle(fontSize: 10.5)),
          ),
        if (current.loginMessage != null)
          Padding(
            padding: const EdgeInsets.only(top: 6),
            child: Text(ltr(current.loginMessage!),
                style: TextStyle(fontSize: 11.5,
                    color: failed ? HorizonPalette.danger : HorizonPalette.textSecondary)),
          ),
        const SizedBox(height: 10),
        Row(children: <Widget>[
          Expanded(child: _OutlineButton(s.t('copyCode'), Icons.copy_rounded,
              () => ActionRunner(ref, context).run(HorizonAction.CopyLoginCode))),
          const SizedBox(width: 8),
          Expanded(child: _OutlineButton(s.t('openLink'), Icons.open_in_new_rounded,
              () => ActionRunner(ref, context).run(HorizonAction.LoginLink))),
        ]),
        if (failed) ...<Widget>[
          const SizedBox(height: 8),
          GradientButton(label: s.t('retry'), icon: Icons.refresh_rounded, height: 40,
              onTap: () => ActionRunner(ref, context).run(HorizonAction.SignIn)),
        ],
      ]),
    );
  }
}

class _StageTag extends StatelessWidget {
  const _StageTag(this.label);
  final String label;
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
        decoration: BoxDecoration(
          color: HorizonPalette.secondary.withValues(alpha: 0.2),
          borderRadius: BorderRadius.circular(HorizonRadii.pill),
        ),
        child: Text(label, style: const TextStyle(fontSize: 10)),
      );
}

class _AccountGroup extends StatelessWidget {
  const _AccountGroup({
    required this.title, required this.accounts, required this.empty,
    required this.s, required this.onSelect, required this.onDelete,
  });
  final String title;
  final List<LauncherAccount> accounts;
  final String empty;
  final HorizonStrings s;
  final ValueChanged<LauncherAccount> onSelect;
  final ValueChanged<LauncherAccount> onDelete;

  @override
  Widget build(BuildContext context) => Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Text(title, style: const TextStyle(fontSize: 12.5, fontWeight: FontWeight.w700)),
          const SizedBox(height: 6),
          if (accounts.isEmpty)
            Padding(
              padding: const EdgeInsets.all(10),
              child: Text(empty, style: const TextStyle(fontSize: 11, color: HorizonPalette.textSecondary)),
            )
          else
            ...accounts.map((LauncherAccount a) => Padding(
                  padding: const EdgeInsets.only(bottom: 6),
                  child: HorizonCard(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                    child: Row(children: <Widget>[
                      Icon(a.isOffline ? Icons.person_outline_rounded : Icons.cloud_rounded,
                          size: 20, color: a.selected ? HorizonPalette.success : HorizonPalette.textSecondary),
                      const SizedBox(width: 10),
                      Expanded(child: Text(ltr(a.name), maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w600))),
                      if (a.selected) _StageTag(s.t('defaultAccount')),
                      IconButton(
                        tooltip: s.t('select'),
                        onPressed: () => onSelect(a),
                        icon: const Icon(Icons.check_circle_outline_rounded, size: 19),
                      ),
                      IconButton(
                        tooltip: s.t('delete'),
                        onPressed: () => onDelete(a),
                        icon: const Icon(Icons.delete_outline_rounded, size: 19, color: HorizonPalette.danger),
                      ),
                    ]),
                  ),
                )),
        ],
      );
}

/// A chooser opens a real value list; applying a value uses the matching
/// `Choose*` Rust action, never the display-only opener.
HorizonAction _applyAction(HorizonAction action) => switch (action) {
      HorizonAction.Renderer => HorizonAction.ChooseRenderer,
      HorizonAction.RenderScale => HorizonAction.ChooseRenderScale,
      HorizonAction.Orientation => HorizonAction.ChooseOrientation,
      HorizonAction.Aspect => HorizonAction.ChooseAspect,
      HorizonAction.DisplayHz => HorizonAction.ChooseDisplayHz,
      HorizonAction.Profile => HorizonAction.ChooseProfile,
      HorizonAction.InputHost => HorizonAction.ChooseInputHost,
      _ => action,
    };

void showChooserSheet(BuildContext context, WidgetRef ref, HorizonAction action) {
  showModalBottomSheet<void>(
    context: context,
    useSafeArea: true,
    showDragHandle: true,
    builder: (BuildContext ctx) => AnimatedSheet(child: _ChooserSheet(action: action, ref: ref)),
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
                      ActionRunner(ref, context).run(_applyAction(action),
                          args: <String, Object?>{'value': option});
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
  final HorizonStrings s = ref.read(stringsProvider);
  showModalBottomSheet<void>(
    context: context,
    useSafeArea: true,
    showDragHandle: true,
    builder: (BuildContext ctx) => AnimatedSheet(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(18, 8, 18, 24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            SectionTitle('${s.t('importContent')} · ${kind.name}'),
            Text(
              s.arabic
                  ? 'تُختار الملفات عبر Android SAF ويُتحقّق منها في المضيف الأصلي قبل دخول مجلد المحتوى.'
                  : 'Files are picked with the Android Storage Access Framework and validated by the native host before they enter the content folder.',
              style: const TextStyle(fontSize: 12, color: HorizonPalette.textSecondary),
            ),
            const SizedBox(height: 16),
            GradientButton(
              label: s.t('importFile'),
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
    ),
  );
}

/// Real project details from the source response (no invented data).
void showProjectDetails(BuildContext context, HorizonStrings s, CatalogProject p) {
  showModalBottomSheet<void>(
    context: context,
    isScrollControlled: true,
    useSafeArea: true,
    showDragHandle: true,
    builder: (BuildContext ctx) => AnimatedSheet(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(18, 8, 18, 24),
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: <Widget>[
              SectionTitle(s.t('projectDetails'), subtitle: p.title),
              if (p.description != null && p.description!.isNotEmpty)
                Padding(
                  padding: const EdgeInsets.only(bottom: 10),
                  child: Text(p.description!, style: const TextStyle(fontSize: 12)),
                ),
              if (p.author != null) _Detail(s.t('author'), ltr(p.author!)),
              if (p.source != null) _Detail(s.t('sourceUpdates'), ltr(p.source!)),
              if (p.downloads != null) _Detail(s.t('downloads'), ltr('${p.downloads!}')),
              if (p.compatibility != null) _Detail(s.t('compatibility'), p.compatibility!),
              if (p.versions.isNotEmpty)
                _Detail(s.t('supportedVersions'), ltr(p.versions.take(6).join(', '))),
              if (p.loaders.isNotEmpty)
                _Detail(s.t('loaders'), ltr(p.loaders.take(6).join(', '))),
              if (p.pageUrl != null) _Detail(s.t('details'), ltr(p.pageUrl!)),
            ],
          ),
        ),
      ),
    ),
  );
}

class _Detail extends StatelessWidget {
  const _Detail(this.label, this.value);
  final String label;
  final String value;
  @override
  Widget build(BuildContext context) => Padding(
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: <Widget>[
          SizedBox(width: 110, child: Text(label, style: const TextStyle(fontSize: 11,
              color: HorizonPalette.textSecondary))),
          Expanded(child: Text(value, style: const TextStyle(fontSize: 11.5))),
        ]),
      );
}

class _OutlineButton extends StatelessWidget {
  const _OutlineButton(this.label, this.icon, this.onTap);
  final String label;
  final IconData icon;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => HorizonPressable(
        onTap: onTap,
        child: Container(
          padding: const EdgeInsets.symmetric(vertical: 13),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(HorizonRadii.control),
            border: Border.all(color: HorizonPalette.border),
            color: HorizonPalette.card,
          ),
          child: adaptiveIconLabel(
            text: label,
            icon: icon,
            iconSize: 18,
            iconColor: HorizonPalette.textSecondary,
            gap: 8,
            style: const TextStyle(fontSize: 14),
          ),
        ),
      );
}
