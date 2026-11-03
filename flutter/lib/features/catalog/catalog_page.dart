import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/bridge/actions.dart';
import '../../core/bridge/models.dart';
import '../../core/state/providers.dart';
import '../../core/theme/horizon_strings.dart';
import '../../core/theme/horizon_theme.dart';
import '../../core/widgets/horizon_widgets.dart';
import '../dialogs/dialogs.dart';
import '../../navigation/app_shell.dart';

/// One real implementation for Mods / Packs / Shaders: the Rust host only has
/// three variants of the same catalog page, so Flutter mirrors that structure.
class CatalogPage extends ConsumerStatefulWidget {
  const CatalogPage({super.key, required this.kind});

  final ContentKind kind;

  @override
  ConsumerState<CatalogPage> createState() => _CatalogPageState();
}

class _CatalogPageState extends ConsumerState<CatalogPage> {
  final TextEditingController _query = TextEditingController();

  @override
  void dispose() {
    _query.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final HorizonStrings s = ref.watch(stringsProvider);
    final HorizonMetrics m = HorizonMetrics.of(context);
    final HorizonAction sourceAction = widget.kind == ContentKind.Shaders
        ? HorizonAction.ShaderSource
        : HorizonAction.ModSource;
    return AsyncSnapshotView(
      state: ref.watch(snapshotProvider),
      strings: s,
      onRetry: () => ref.invalidate(snapshotProvider),
      builder: (LauncherSnapshot d) => SingleChildScrollView(
        padding: EdgeInsets.fromLTRB(m.horizontal, m.padding.top + 12, m.horizontal, 20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            SectionTitle(_title(s), subtitle: _subtitle(s)),
            Row(
              children: <Widget>[
                Expanded(child: HorizonChip(label: s.t('mods'), active: widget.kind == ContentKind.Mods, onTap: () => ref.read(navIndexProvider.notifier).state = 1)),
                const SizedBox(width: 8),
                Expanded(child: HorizonChip(label: s.t('packs'), active: widget.kind == ContentKind.Packs, onTap: () => ref.read(navIndexProvider.notifier).state = 2)),
                const SizedBox(width: 8),
                Expanded(child: HorizonChip(label: s.t('shaders'), active: widget.kind == ContentKind.Shaders, onTap: () => openShaders(ref))),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              children: <Widget>[
                Expanded(
                  child: TextField(
                    controller: _query,
                    textInputAction: TextInputAction.search,
                    onSubmitted: (String value) => ActionRunner(ref, context).run(
                      HorizonAction.Search,
                      args: <String, Object?>{'query': value, 'kind': widget.kind.name},
                    ),
                    decoration: InputDecoration(
                      hintText: '${s.t('search')} …',
                      prefixIcon: const Icon(Icons.search_rounded, size: 20),
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                SizedBox(
                  width: 48,
                  height: 48,
                  child: HorizonCard(
                    padding: EdgeInsets.zero,
                    onTap: () => showChooserSheet(context, ref, sourceAction),
                    child: const Icon(Icons.tune_rounded, size: 20),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              children: <Widget>[
                Expanded(child: HorizonChip(label: 'Horizon', icon: Icons.change_history_rounded, active: _activeSource(d) == 'Horizon', onTap: () => _pickSource(ref, context, sourceAction, 'Horizon'))),
                const SizedBox(width: 8),
                Expanded(child: HorizonChip(label: 'Modrinth', icon: Icons.public_rounded, active: _activeSource(d) == 'Modrinth', onTap: () => _pickSource(ref, context, sourceAction, 'Modrinth'))),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              children: <Widget>[
                Expanded(child: _SmallAction(s.t('openFolder'), Icons.folder_open_rounded, () => ActionRunner(ref, context).run(HorizonAction.OpenFolder, args: <String, Object?>{'kind': widget.kind.name}))),
                const SizedBox(width: 8),
                Expanded(child: _SmallAction(s.t('importFile'), Icons.upload_file_rounded, () => showImportDialog(context, ref, widget.kind))),
              ],
            ),
            if (d.status != null) ...<Widget>[
              const SizedBox(height: 12),
              Text(d.status!, style: const TextStyle(fontSize: 11.5, color: HorizonPalette.textSecondary)),
            ],
            const SizedBox(height: 12),
            if (d.results.isEmpty)
              HorizonCard(
                child: EmptyState(title: s.t('noResults'), hint: s.t('noResultsHint'), icon: Icons.search_off_rounded),
              )
            else
              ...d.results.map((CatalogProject p) => Padding(
                    padding: const EdgeInsets.only(bottom: 10),
                    child: HorizonCard(
                      child: Row(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: <Widget>[
                          Container(
                            width: 48, height: 48,
                            decoration: BoxDecoration(
                              color: HorizonPalette.secondary.withValues(alpha: 0.18),
                              borderRadius: BorderRadius.circular(12),
                            ),
                            child: const Icon(Icons.image_outlined, color: HorizonPalette.accent),
                          ),
                          const SizedBox(width: 12),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: <Widget>[
                                Text(p.title, maxLines: 1, overflow: TextOverflow.ellipsis,
                                    style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w700)),
                                if (p.description != null)
                                  Text(p.description!, maxLines: 2, overflow: TextOverflow.ellipsis,
                                      style: const TextStyle(fontSize: 11.5, color: HorizonPalette.textSecondary)),
                                const SizedBox(height: 4),
                                Wrap(spacing: 8, children: <Widget>[
                                  if (p.version != null) _Tag('Works · ${p.version!}'),
                                  if (p.source != null) _Tag(p.source!),
                                  if (p.compatibility != null) _Tag(p.compatibility!),
                                ]),
                              ],
                            ),
                          ),
                          const SizedBox(width: 8),
                          HorizonPressable(
                            onTap: () => ActionRunner(ref, context).run(
                              HorizonAction.Install,
                              args: <String, Object?>{'index': p.index, 'kind': widget.kind.name},
                            ),
                            child: Container(
                              padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 9),
                              decoration: BoxDecoration(
                                gradient: HorizonPalette.primaryGradient,
                                borderRadius: BorderRadius.circular(HorizonRadii.pill),
                              ),
                              child: Text(s.t('install'), style: const TextStyle(fontSize: 12, color: Colors.white, fontWeight: FontWeight.w700)),
                            ),
                          ),
                        ],
                      ),
                    ),
                  )),
            const SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: <Widget>[
                _SmallAction('‹', Icons.chevron_left_rounded, () => ActionRunner(ref, context).run(HorizonAction.Previous)),
                _SmallAction('›', Icons.chevron_right_rounded, () => ActionRunner(ref, context).run(HorizonAction.Next)),
              ],
            ),
          ],
        ),
      ),
    );
  }

  String _title(HorizonStrings s) {
    switch (widget.kind) {
      case ContentKind.Mods: return s.t('mods');
      case ContentKind.Packs: return 'Resource Packs';
      case ContentKind.Shaders: return s.t('shaders');
    }
  }

  String _subtitle(HorizonStrings s) {
    switch (widget.kind) {
      case ContentKind.Mods: return 'Native .hmod packages • install, enable or remove';
      case ContentKind.Packs: return 'Verified downloads • enable in Installed';
      case ContentKind.Shaders: return 'Lighting, shadows and visual effects';
    }
  }

  String? _activeSource(LauncherSnapshot d) =>
      d.setting(widget.kind == ContentKind.Shaders ? SettingKeys.shaderSource : SettingKeys.modSource) as String?;

  void _pickSource(WidgetRef ref, BuildContext context, HorizonAction action, String value) {
    ActionRunner(ref, context).run(action, args: <String, Object?>{'value': value});
  }
}

class _Tag extends StatelessWidget {
  const _Tag(this.label);
  final String label;

  @override
  Widget build(BuildContext context) => Container(
        margin: const EdgeInsets.only(top: 4),
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
        decoration: BoxDecoration(
          color: HorizonPalette.secondary.withValues(alpha: 0.15),
          borderRadius: BorderRadius.circular(HorizonRadii.pill),
        ),
        child: Text(label, maxLines: 1, overflow: TextOverflow.ellipsis,
            style: const TextStyle(fontSize: 10, color: HorizonPalette.textSecondary)),
      );
}

class _SmallAction extends StatelessWidget {
  const _SmallAction(this.label, this.icon, this.onTap);
  final String label;
  final IconData icon;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => HorizonCard(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 11),
        onTap: onTap,
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            Icon(icon, size: 17, color: HorizonPalette.accent),
            const SizedBox(width: 6),
            Flexible(child: Text(label, maxLines: 1, overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 12))),
          ],
        ),
      );
}
