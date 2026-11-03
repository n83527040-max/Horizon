import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/bridge/actions.dart';
import '../../core/bridge/models.dart';
import '../../core/state/providers.dart';
import '../../core/theme/horizon_strings.dart';
import '../../core/theme/horizon_motion.dart';
import '../../core/theme/horizon_theme.dart';
import '../../core/widgets/horizon_widgets.dart';
import '../dialogs/dialogs.dart';
import '../../navigation/app_shell.dart';

class HomePage extends ConsumerWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final HorizonStrings s = ref.watch(stringsProvider);
    final HorizonMetrics m = HorizonMetrics.of(context);
    return AsyncSnapshotView(
      state: ref.watch(snapshotProvider),
      strings: s,
      onRetry: () => ref.invalidate(snapshotProvider),
      builder: (LauncherSnapshot d) => SingleChildScrollView(
        padding: EdgeInsets.fromLTRB(m.horizontal, m.padding.top + 12, m.horizontal, 20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            _Header(s: s, d: d, onSettings: () => ref.read(navIndexProvider.notifier).state = 4),
            SizedBox(height: m.isTablet ? 20 : 14),
            StaggeredEntrance(
              child: _Hero(height: m.heroHeight, gameVersion: d.gameVersion),
            ),
            const SizedBox(height: 14),
            GradientButton(
              label: s.t('play'),
              icon: Icons.play_arrow_rounded,
              onTap: () => ActionRunner(ref, context).run(HorizonAction.Play),
            ),
            const SizedBox(height: 14),
            Row(
              children: <Widget>[
                Expanded(child: _InfoCard(label: s.t('version'), value: d.gameVersion, icon: Icons.extension_rounded, onTap: () {})),
                const SizedBox(width: 12),
                Expanded(
                  child: _InfoCard(
                    label: s.t('profile'),
                    value: d.profileName ?? (d.accountStatus != null ? ltr(d.accountStatus!) : null) ?? '—',
                    icon: Icons.person_rounded,
                    onTap: () => showAccountsSheet(context, ref, d),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 18),
            SectionTitle(s.t('manage')),
            GridView.count(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              crossAxisCount: 2,
              mainAxisSpacing: 12,
              crossAxisSpacing: 12,
              childAspectRatio: 2.15,
              children: <Widget>[
                _ManageCard(Icons.extension_rounded, s.t('mods'), '${d.count(ContentKind.Mods)}', () => ref.read(navIndexProvider.notifier).state = 1),
                _ManageCard(Icons.layers_rounded, s.t('packs'), '${d.count(ContentKind.Packs)}', () => ref.read(navIndexProvider.notifier).state = 2),
                _ManageCard(Icons.auto_awesome_rounded, s.t('shaders'), '${d.count(ContentKind.Shaders)}', () => openShaders(ref)),
                _ManageCard(Icons.inventory_2_rounded, s.t('installed'), '${d.totalInstalled}', () => ref.read(navIndexProvider.notifier).state = 3),
              ],
            ),
            const SizedBox(height: 18),
            SectionTitle(s.t('performance')),
            _PerformanceCard(d: d, s: s),
          ],
        ),
      ),
    );
  }
}

class _Header extends StatelessWidget {
  const _Header({required this.s, required this.d, required this.onSettings});
  final HorizonStrings s;
  final LauncherSnapshot d;
  final VoidCallback onSettings;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: <Widget>[
        Container(
          width: 38,
          height: 38,
          decoration: BoxDecoration(
            gradient: HorizonPalette.primaryGradient,
            borderRadius: BorderRadius.circular(12),
          ),
          child: const Icon(Icons.change_history_rounded, color: Colors.white, size: 22),
        ),
        const SizedBox(width: 10),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              Text(s.t('appName'), style: const TextStyle(fontSize: 19, fontWeight: FontWeight.w800, letterSpacing: 1.5)),
              Text(s.t('appSubtitle'), maxLines: 1, overflow: TextOverflow.ellipsis,
                  style: const TextStyle(fontSize: 10.5, color: HorizonPalette.textSecondary)),
            ],
          ),
        ),
        if (d.appVersion.isNotEmpty)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
            decoration: BoxDecoration(
              color: HorizonPalette.primary.withValues(alpha: 0.22),
              borderRadius: BorderRadius.circular(HorizonRadii.pill),
            ),
            child: Text('v${d.appVersion}', style: const TextStyle(fontSize: 11, color: Colors.white)),
          ),
        const SizedBox(width: 8),
        IconButton(onPressed: onSettings, icon: const Icon(Icons.settings_rounded)),
      ],
    );
  }
}

class _Hero extends StatelessWidget {
  const _Hero({required this.height, required this.gameVersion});
  final double height;
  final String gameVersion;

  @override
  Widget build(BuildContext context) {
    return ClipRRect(
      borderRadius: BorderRadius.circular(HorizonRadii.card),
      child: SizedBox(
        height: height,
        width: double.infinity,
        child: Stack(
          fit: StackFit.expand,
          children: <Widget>[
            Image.asset('assets/hero.png', fit: BoxFit.cover,
                errorBuilder: (_, __, ___) => Container(color: HorizonPalette.card)),
            const DecoratedBox(decoration: BoxDecoration(gradient: HorizonPalette.heroGradient)),
            Positioned(
              left: 18,
              bottom: 16,
              right: 18,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: <Widget>[
                  const Text('HORIZON', style: TextStyle(fontSize: 26, fontWeight: FontWeight.w900, color: Colors.white, letterSpacing: 1)),
                  Text('Minecraft Java Edition${gameVersion.isEmpty ? '' : ' • $gameVersion'}',
                      maxLines: 1, overflow: TextOverflow.ellipsis,
                      style: const TextStyle(fontSize: 12, color: Colors.white70)),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _InfoCard extends StatelessWidget {
  const _InfoCard({required this.label, required this.value, required this.icon, required this.onTap});
  final String label;
  final String value;
  final IconData icon;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return HorizonCard(
      onTap: onTap,
      padding: const EdgeInsets.all(12),
      child: Row(
        children: <Widget>[
          Icon(icon, color: HorizonPalette.accent, size: 22),
          const SizedBox(width: 10),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                Text(label, maxLines: 1, overflow: TextOverflow.ellipsis,
                    style: const TextStyle(fontSize: 10.5, color: HorizonPalette.textSecondary)),
                Text(value, maxLines: 1, overflow: TextOverflow.ellipsis,
                    style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w700)),
              ],
            ),
          ),
          const Icon(Icons.keyboard_arrow_down_rounded, size: 18, color: HorizonPalette.textSecondary),
        ],
      ),
    );
  }
}

class _ManageCard extends StatelessWidget {
  const _ManageCard(this.icon, this.label, this.count, this.onTap);
  final IconData icon;
  final String label;
  final String count;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return HorizonCard(
      onTap: onTap,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
      child: Row(
        children: <Widget>[
          Container(
            width: 34, height: 34,
            decoration: BoxDecoration(
              color: HorizonPalette.primary.withValues(alpha: 0.18),
              borderRadius: BorderRadius.circular(10),
            ),
            child: Icon(icon, size: 18, color: HorizonPalette.primary),
          ),
          const SizedBox(width: 10),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                Text(label, maxLines: 1, overflow: TextOverflow.ellipsis,
                    style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w700)),
                Text(count, maxLines: 1, overflow: TextOverflow.ellipsis,
                    style: const TextStyle(fontSize: 10, color: HorizonPalette.textSecondary)),
              ],
            ),
          ),
          const Icon(Icons.chevron_right_rounded, size: 18, color: HorizonPalette.textSecondary),
        ],
      ),
    );
  }
}

class _PerformanceCard extends StatelessWidget {
  const _PerformanceCard({required this.d, required this.s});
  final LauncherSnapshot d;
  final HorizonStrings s;

  @override
  Widget build(BuildContext context) {
    final int? used = d.ramUsedMb;
    final int? total = d.ramTotalMb;
    final double? fraction = (used != null && total != null && total > 0) ? (used / total).clamp(0.0, 1.0) : null;
    return HorizonCard(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Row(
            children: <Widget>[
              const Icon(Icons.speed_rounded, color: HorizonPalette.primary, size: 18),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  used != null && total != null
                      ? ltr('$used MB / ${_gib(total)}')
                      : s.t('performance'),
                  style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w700),
                ),
              ),
            ],
          ),
          const SizedBox(height: 10),
          ClipRRect(
            borderRadius: BorderRadius.circular(6),
            child: LinearProgressIndicator(
              value: fraction,
              minHeight: 6,
              backgroundColor: HorizonPalette.border,
              valueColor: const AlwaysStoppedAnimation<Color?>(HorizonPalette.primary),
            ),
          ),
          const SizedBox(height: 12),
          Wrap(
            spacing: 14,
            runSpacing: 8,
            children: <Widget>[
              if (d.graphicsApi != null) MetricTile(label: s.t('graphicsApi'), value: d.graphicsApi!, icon: Icons.memory_rounded),
              if (d.renderScalePercent != null) MetricTile(label: s.t('resolution'), value: ltr('${d.renderScalePercent}%'), icon: Icons.aspect_ratio_rounded),
              if (d.ramPeakMb != null) MetricTile(label: s.t('ramPeak'), value: ltr('${d.ramPeakMb} MB'), icon: Icons.trending_up_rounded),
            ],
          ),
        ],
      ),
    );
  }

  static String _gib(int mb) => mb >= 1024 ? '${(mb / 1024).toStringAsFixed(1)} GB' : '$mb MB';
}
