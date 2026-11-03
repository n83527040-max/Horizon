import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/bridge/actions.dart';
import '../../core/bridge/models.dart';
import '../../core/state/providers.dart';
import '../../core/theme/horizon_strings.dart';
import '../../core/theme/horizon_theme.dart';
import '../../core/widgets/horizon_widgets.dart';
import '../dialogs/dialogs.dart';

class SettingsPage extends ConsumerWidget {
  const SettingsPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final HorizonStrings s = ref.watch(stringsProvider);
    final HorizonMetrics m = HorizonMetrics.of(context);
    return AsyncSnapshotView(
      state: ref.watch(snapshotProvider),
      strings: s,
      onRetry: () => ref.invalidate(snapshotProvider),
      builder: (LauncherSnapshot d) => _RefScope(
        ref: ref,
        child: SingleChildScrollView(
        padding: EdgeInsets.fromLTRB(m.horizontal, m.padding.top + 12, m.horizontal, 20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            SectionTitle(s.t('settings'), subtitle: 'Graphics, performance, accounts and app'),
            _Group(s.t('graphics'), <Widget>[
              _Row(s.t('rendering'), _value(d, SettingKeys.renderer), HorizonAction.Renderer),
              _Row(s.t('resolution'), _percent(d), HorizonAction.RenderScale),
              _Row(s.t('aspect'), _value(d, SettingKeys.aspect), HorizonAction.Aspect),
              _Row(s.t('refreshRate'), _value(d, SettingKeys.displayHz), HorizonAction.DisplayHz),
              _Switch(s.t('immersive'), d.setting(SettingKeys.immersive) == true, HorizonAction.Immersive),
              _Switch(s.t('cutout'), d.setting(SettingKeys.avoidCutout) == true, HorizonAction.Cutout),
            ]),
            _Group(s.t('performance'), <Widget>[
              _Switch(s.t('perfHints'), d.setting(SettingKeys.performanceHints) == true, HorizonAction.PerformanceHints),
              _Row(s.t('gpuProfiling'), _onOff(d.setting(SettingKeys.gpuProfile) == true), HorizonAction.GpuProfile),
              _Switch(s.t('recording'), d.setting(SettingKeys.recordPerformance) == true, HorizonAction.RecordPerformance),
              _Row(s.t('ramUsage'), d.ramUsedMb == null ? null : '${d.ramUsedMb} MB', null),
              _Row(s.t('graphicsApi'), d.graphicsApi, null),
              _Row(s.t('diagnostics'), 'Export one ZIP to share', HorizonAction.ExportDiagnostics),
            ]),
            _Group(s.t('accounts'), <Widget>[
              _Row('Microsoft / Xbox account', d.accountStatus ?? d.profileName ?? '—', HorizonAction.Account),
              _Row(s.t('offlineName'), d.profileName, HorizonAction.Field),
              _Row(s.t('orientation'), _value(d, SettingKeys.orientation), HorizonAction.Orientation),
              _Row(s.t('server'), _value(d, SettingKeys.server), HorizonAction.Field),
              _Row('Profile', _value(d, SettingKeys.profile), HorizonAction.Profile),
            ]),
            _Group(s.t('application'), <Widget>[
              _Choice(
                s.t('theme'),
                <String, String>{'system': s.t('system'), 'dark': s.t('dark'), 'light': s.t('light')},
                (d.setting(SettingKeys.theme) as String?) ?? 'system',
                (String value) => ActionRunner(ref, context).run(
                      HorizonAction.Field,
                      args: <String, Object?>{'name': SettingKeys.theme, 'value': value, 'changed': true},
                    ),
              ),
              _Choice(
                s.t('language'),
                <String, String>{'en': 'English', 'ar': 'العربية'},
                (d.setting(SettingKeys.language) as String?) ?? 'en',
                (String value) => ActionRunner(ref, context).run(
                      HorizonAction.Field,
                      args: <String, Object?>{'name': SettingKeys.language, 'value': value, 'changed': true},
                    ),
              ),
              _Row(s.t('storage'), _bytes(d.storageBytes), null),
              if (d.contentBytes.isNotEmpty)
                Padding(
                  padding: const EdgeInsets.only(top: 6),
                  child: Wrap(
                    spacing: 10,
                    runSpacing: 6,
                    children: d.contentBytes.entries
                        .map((MapEntry<String, int> e) => _Tag('${e.key}: ${_bytes(e.value)}'))
                        .toList(),
                  ),
                ),
              _Row(s.t('cache'), _bytes(d.cacheBytes), null),
              Padding(
                padding: const EdgeInsets.only(top: 10),
                child: GradientButton(
                  label: s.t('clearCache'),
                  icon: Icons.cleaning_services_rounded,
                  height: 44,
                  onTap: () => ActionRunner(ref, context).run(
                        HorizonAction.Field,
                        args: <String, Object?>{'name': 'clearCache', 'changed': true},
                      ),
                ),
              ),
              _Row(s.t('appVersion'), d.appVersion.isEmpty ? null : 'v${d.appVersion}', null),
              _Row(s.t('about'), 'Minecraft Java ${d.gameVersion} • Rust engine', HorizonAction.Repository),
              _Row(s.t('sourceUpdates'), 'Open the repository', HorizonAction.Repository),
            ]),
            const SizedBox(height: 10),
            Text(
              'Flutter host · engine, Vulkan, accounts, mods and files stay native. '
              'Values shown here are read from the real launcher configuration.',
              textAlign: TextAlign.center,
              style: const TextStyle(fontSize: 10.5, color: HorizonPalette.textSecondary),
            ),
          ],
        ),
      ),
      ),
    );
  }

  static String? _value(LauncherSnapshot d, String key) {
    final Object? v = d.setting(key);
    if (v == null) return null;
    if (v is bool) return v ? 'On' : 'Off';
    final String text = v.toString();
    return text.isEmpty ? null : text;
  }

  static String? _percent(LauncherSnapshot d) =>
      d.renderScalePercent == null ? null : '${d.renderScalePercent}%';

  static String _onOff(bool value) => value ? 'On' : 'Off';

  static String? _bytes(int? value) {
    if (value == null) return null;
    if (value >= 1 << 30) return '${(value / (1 << 30)).toStringAsFixed(2)} GB';
    if (value >= 1 << 20) return '${(value / (1 << 20)).toStringAsFixed(1)} MB';
    if (value >= 1 << 10) return '${(value / (1 << 10)).toStringAsFixed(0)} KB';
    return '$value B';
  }
}

class _Group extends StatelessWidget {
  const _Group(this.title, this.rows);
  final String title;
  final List<Widget> rows;

  @override
  Widget build(BuildContext context) => Padding(
        padding: const EdgeInsets.only(bottom: 14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            Padding(
              padding: const EdgeInsets.only(bottom: 8, top: 4),
              child: Text(title, style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w800, color: HorizonPalette.accent)),
            ),
            HorizonCard(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              child: Column(children: rows),
            ),
          ],
        ),
      );
}

class _Row extends StatelessWidget {
  const _Row(this.label, this.value, this.action);
  final String label;
  final String? value;
  final HorizonAction? action;

  @override
  Widget build(BuildContext context) => ListTile(
        dense: true,
        contentPadding: EdgeInsets.zero,
        title: Text(label, maxLines: 1, overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 13.5, fontWeight: FontWeight.w600)),
        subtitle: value == null
            ? null
            : Text(value!, maxLines: 1, overflow: TextOverflow.ellipsis,
                style: const TextStyle(fontSize: 11, color: HorizonPalette.textSecondary)),
        trailing: action == null ? null : const Icon(Icons.chevron_right_rounded, size: 18, color: HorizonPalette.textSecondary),
        onTap: action == null
            ? null
            : () {
                final WidgetRef ref = _refOf(context);
                final List<String> options =
                    ref.read(snapshotProvider).valueOrNull?.choices[action!.name] ??
                        const <String>[];
                if (options.isNotEmpty) {
                  showChooserSheet(context, ref, action!);
                } else {
                  ActionRunner(ref, context).run(action!,
                      args: <String, Object?>{'name': label});
                }
              },
      );

  // The row is built inside a ConsumerWidget subtree; find the nearest ref via
  // an inherited marker installed by the page (see _RefScope below).
  static WidgetRef _refOf(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<_RefScope>()!.ref;
}

class _Switch extends StatelessWidget {
  const _Switch(this.label, this.value, this.action);
  final String label;
  final bool value;
  final HorizonAction action;

  @override
  Widget build(BuildContext context) {
    final WidgetRef ref = _Row._refOf(context);
    return SwitchListTile(
      dense: true,
      contentPadding: EdgeInsets.zero,
      title: Text(label, maxLines: 1, overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 13.5, fontWeight: FontWeight.w600)),
      value: value,
      onChanged: (bool _) => ActionRunner(ref, context).run(action),
    );
  }
}

class _Choice extends StatelessWidget {
  const _Choice(this.label, this.options, this.current, this.onChanged);
  final String label;
  final Map<String, String> options;
  final String current;
  final ValueChanged<String> onChanged;

  @override
  Widget build(BuildContext context) => Padding(
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Text(label, style: const TextStyle(fontSize: 13.5, fontWeight: FontWeight.w600)),
            const SizedBox(height: 8),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: options.entries
                  .map((MapEntry<String, String> e) => HorizonChip(
                        label: e.value,
                        active: e.key == current,
                        onTap: () => onChanged(e.key),
                      ))
                  .toList(),
            ),
          ],
        ),
      );
}

class _Tag extends StatelessWidget {
  const _Tag(this.label);
  final String label;

  @override
  Widget build(BuildContext _) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          color: HorizonPalette.secondary.withValues(alpha: 0.15),
          borderRadius: BorderRadius.circular(HorizonRadii.pill),
        ),
        child: Text(label, style: const TextStyle(fontSize: 10, color: HorizonPalette.textSecondary)),
      );
}

/// Supplies the current WidgetRef to stateless rows without prop-drilling.
class _RefScope extends InheritedWidget {
  const _RefScope({required this.ref, required super.child});
  final WidgetRef ref;

  @override
  bool updateShouldNotify(_RefScope oldWidget) => oldWidget.ref != ref;
}
