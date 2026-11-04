import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/bridge/actions.dart';
import '../../core/bridge/models.dart';
import '../../core/state/providers.dart';
import '../../core/theme/horizon_motion.dart';
import '../../core/theme/horizon_strings.dart';
import '../../core/theme/horizon_theme.dart';
import '../../core/widgets/horizon_widgets.dart';
import '../dialogs/dialogs.dart';
import '../../navigation/app_shell.dart';

/// One real implementation for Mods / Packs / Shaders. Results, authors,
/// versions, loaders, icons and compatibility all come from the real Rust
/// catalog (Modrinth / Horizon). Errors are shown with their real reason.
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
        padding: EdgeInsets.fromLTRB(m.horizontal, m.padding.top + 12, m.horizontal, 24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            StaggeredEntrance(
              child: SectionTitle(_title(s), subtitle: _subtitle(s)),
            ),
            StaggeredEntrance(
              index: 1,
              child: Row(children: <Widget>[
                Expanded(child: HorizonChip(label: s.t('mods'), active: widget.kind == ContentKind.Mods,
                    onTap: () => ref.read(navIndexProvider.notifier).state = 1)),
                const SizedBox(width: 8),
                Expanded(child: HorizonChip(label: s.t('packs'), active: widget.kind == ContentKind.Packs,
                    onTap: () => ref.read(navIndexProvider.notifier).state = 2)),
                const SizedBox(width: 8),
                Expanded(child: HorizonChip(label: s.t('shaders'), active: widget.kind == ContentKind.Shaders,
                    onTap: () => openShaders(ref))),
              ]),
            ),
            const SizedBox(height: 12),
            StaggeredEntrance(
              index: 2,
              child: Row(children: <Widget>[
                Expanded(
                  child: TextField(
                    controller: _query,
                    textInputAction: TextInputAction.search,
                    onSubmitted: (String value) => _search(ref, context, value, d),
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
              ]),
            ),
            const SizedBox(height: 12),
            StaggeredEntrance(
              index: 3,
              child: Row(children: <Widget>[
                Expanded(child: HorizonChip(label: 'Horizon', icon: Icons.change_history_rounded,
                    active: _activeSource(d) == 'Horizon',
                    onTap: () => _pickSource(ref, context, sourceAction, 'Horizon'))),
                const SizedBox(width: 8),
                Expanded(child: HorizonChip(label: 'Modrinth', icon: Icons.public_rounded,
                    active: _activeSource(d) == 'Modrinth',
                    onTap: () => _pickSource(ref, context, sourceAction, 'Modrinth'))),
              ]),
            ),
            const SizedBox(height: 12),
            _Filters(s: s, d: d, kind: widget.kind, ref: ref, context: context),
            const SizedBox(height: 12),
            Row(children: <Widget>[
              Expanded(child: _SmallAction(s.t('openFolder'), Icons.folder_open_rounded,
                  () => ActionRunner(ref, context).run(HorizonAction.OpenFolder,
                      args: <String, Object?>{'kind': widget.kind.name}))),
              const SizedBox(width: 8),
              Expanded(child: _SmallAction(s.t('importFile'), Icons.upload_file_rounded,
                  () => showImportDialog(context, ref, widget.kind))),
            ]),
            if (d.progress != null) ...<Widget>[
              const SizedBox(height: 12),
              AnimatedProgress(value: d.progress!, label: s.t('installing')),
            ],
            const SizedBox(height: 12),
            // REAL error state: never a generic "No results" for network/API errors.
            if (d.hasError && d.results.isNotEmpty)
              Padding(
                padding: const EdgeInsets.only(bottom: 10),
                child: HorizonCard(
                  child: Row(children: <Widget>[
                    const Icon(Icons.error_outline_rounded, color: HorizonPalette.danger, size: 18),
                    const SizedBox(width: 8),
                    Expanded(child: Text(ltr(d.catalogStatus ?? d.status ?? d.errorKind),
                        maxLines: 2, overflow: TextOverflow.ellipsis,
                        style: const TextStyle(fontSize: 11.5))),
                    TextButton(
                      onPressed: () => ActionRunner(ref, context).run(HorizonAction.Field,
                          args: <String, Object?>{'name': 'retry', 'kind': widget.kind.name}),
                      child: Text(s.t('retry')),
                    ),
                  ]),
                ),
              ),
            if (d.hasError && d.results.isEmpty)
              _ErrorCard(s: s, d: d, ref: ref, context: context, kind: widget.kind)
            else if (d.pending && d.results.isEmpty)
              HorizonSkeleton(height: 150, label: s.t('loadingContent'))
            else if (d.results.isEmpty)
              HorizonCard(child: EmptyState(
                title: s.t('noResults'), hint: s.t('noResultsHint'), icon: Icons.search_off_rounded))
            else
              ...d.results.asMap().entries.map((MapEntry<int, CatalogProject> e) =>
                  StaggeredEntrance(
                    index: 4 + e.key,
                    child: Padding(
                      padding: const EdgeInsets.only(bottom: 10),
                      child: _ProjectCard(s: s, p: e.value, d: d, ref: ref, context: context, kind: widget.kind),
                    ),
                  )),
            const SizedBox(height: 12),
            Row(mainAxisAlignment: MainAxisAlignment.spaceBetween, children: <Widget>[
              _SmallAction('', Icons.chevron_left_rounded,
                  () => ActionRunner(ref, context).run(HorizonAction.Previous,
                      args: <String, Object?>{'kind': widget.kind.name})),
              _SmallAction('', Icons.chevron_right_rounded,
                  () => ActionRunner(ref, context).run(HorizonAction.Next,
                      args: <String, Object?>{'kind': widget.kind.name})),
            ]),
          ],
        ),
      ),
    );
  }

  void _search(WidgetRef ref, BuildContext context, String value, LauncherSnapshot d) {
    ActionRunner(ref, context).run(HorizonAction.Search, args: <String, Object?>{
      'query': value,
      'kind': widget.kind.name,
      'version': d.filterVersion,
      'loader': d.filterLoader,
    });
  }

  String _title(HorizonStrings s) {
    switch (widget.kind) {
      case ContentKind.Mods: return s.t('mods');
      case ContentKind.Packs: return s.t('resourcePacks');
      case ContentKind.Shaders: return s.t('shaders');
    }
  }

  String _subtitle(HorizonStrings s) {
    switch (widget.kind) {
      case ContentKind.Mods: return s.t('modsSubtitle');
      case ContentKind.Packs: return s.t('packsSubtitle');
      case ContentKind.Shaders: return s.t('shadersSubtitle');
    }
  }

  String? _activeSource(LauncherSnapshot d) =>
      d.setting(widget.kind == ContentKind.Shaders ? SettingKeys.shaderSource : SettingKeys.modSource) as String?;

  void _pickSource(WidgetRef ref, BuildContext context, HorizonAction action, String value) {
    ActionRunner(ref, context).run(action,
        args: <String, Object?>{'value': value, 'kind': widget.kind.name});
  }
}

/// Real compatibility filters, applied through the JNI dispatcher.
class _Filters extends StatelessWidget {
  const _Filters({required this.s, required this.d, required this.kind, required this.ref, required this.context});

  final HorizonStrings s;
  final LauncherSnapshot d;
  final ContentKind kind;
  final WidgetRef ref;
  final BuildContext context;

  @override
  Widget build(BuildContext _) => Wrap(spacing: 8, runSpacing: 6, children: <Widget>[
    HorizonChip(
      label: d.filterVersion == null ? s.t('allVersions') : ltr('MC ${d.filterVersion!}'),
      icon: Icons.numbers_rounded,
      active: d.filterVersion != null,
      onTap: () => _cycleVersion(),
    ),
    HorizonChip(
      label: d.filterLoader == null ? s.t('allLoaders') : ltr(d.filterLoader!),
      icon: Icons.extension_rounded,
      active: d.filterLoader != null,
      onTap: () => _cycleLoader(),
    ),
  ]);

  void _cycleVersion() {
    const List<String> versions = <String>['', '26.3', '1.21.1'];
    final int i = versions.indexOf(d.filterVersion ?? '');
    ActionRunner(ref, context).run(HorizonAction.Field, args: <String, Object?>{
      'name': 'filter/version', 'value': versions[(i + 1) % versions.length], 'kind': kind.name,
    });
  }

  void _cycleLoader() {
    const List<String> loaders = <String>['', 'fabric', 'neoforge', 'quilt'];
    final int i = loaders.indexOf(d.filterLoader ?? '');
    ActionRunner(ref, context).run(HorizonAction.Field, args: <String, Object?>{
      'name': 'filter/loader', 'value': loaders[(i + 1) % loaders.length], 'kind': kind.name,
    });
  }
}

/// The real reason for a failure plus a real retry action.
class _ErrorCard extends StatelessWidget {
  const _ErrorCard({required this.s, required this.d, required this.ref, required this.context, required this.kind});

  final HorizonStrings s;
  final LauncherSnapshot d;
  final WidgetRef ref;
  final BuildContext context;
  final ContentKind kind;

  @override
  Widget build(BuildContext _) {
    final String detail = d.catalogStatus ?? d.status ?? '';
    final String title;
    String hint;
    switch (d.errorKind) {
      case 'network':
        title = s.t('networkError'); hint = s.t('networkErrorHint'); break;
      case 'api':
        title = s.t('apiError'); hint = s.t('apiErrorHint'); break;
      case 'parse':
        title = s.t('parseError'); hint = s.t('parseErrorHint'); break;
      case 'compatibility':
        title = s.t('compatibilityError'); hint = detail; break;
      case 'storage':
        title = s.t('storageError'); hint = detail; break;
      case 'auth':
        title = s.t('authError'); hint = detail; break;
      default:
        title = s.t('otherError'); hint = detail;
    }
    if (hint.isEmpty) hint = s.t('noResultsHint');
    return HorizonCard(
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: <Widget>[
        Row(children: <Widget>[
          const Icon(Icons.error_outline_rounded, color: HorizonPalette.danger, size: 22),
          const SizedBox(width: 10),
          Expanded(child: Text(title, style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w700))),
          _Tag(ltr(d.errorKind)),
        ]),
        if (hint.isNotEmpty) ...<Widget>[
          const SizedBox(height: 8),
          Text(hint, style: const TextStyle(fontSize: 11.5, color: HorizonPalette.textSecondary)),
        ],
        if (detail.isNotEmpty && detail != hint) ...<Widget>[
          const SizedBox(height: 6),
          Text(s.t('errorDetails'), style: const TextStyle(fontSize: 10, color: HorizonPalette.textSecondary)),
          Text(ltr(detail), style: const TextStyle(fontSize: 10.5, color: HorizonPalette.textSecondary)),
        ],
        const SizedBox(height: 12),
        GradientButton(
          label: s.t('retry'),
          icon: Icons.refresh_rounded,
          height: 42,
          onTap: () => ActionRunner(ref, context).run(HorizonAction.Field,
              args: <String, Object?>{'name': 'retry', 'kind': kind.name}),
        ),
      ]),
    );
  }
}

/// Professional store-style card, entirely from real source data.
class _ProjectCard extends StatelessWidget {
  const _ProjectCard({required this.s, required this.p, required this.d, required this.ref, required this.context, required this.kind});

  final HorizonStrings s;
  final CatalogProject p;
  final LauncherSnapshot d;
  final WidgetRef ref;
  final BuildContext context;
  final ContentKind kind;

  @override
  Widget build(BuildContext _) => HorizonCard(
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: <Widget>[
          Row(crossAxisAlignment: CrossAxisAlignment.start, children: <Widget>[
            _Icon(url: p.iconUrl),
            const SizedBox(width: 12),
            Expanded(
              child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: <Widget>[
                Text(p.title, maxLines: 1, overflow: TextOverflow.ellipsis,
                    style: const TextStyle(fontSize: 14.5, fontWeight: FontWeight.w700)),
                if (p.author != null && p.author!.isNotEmpty)
                  Text('${s.t('author')} ${ltr(p.author!)}', maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(fontSize: 11, color: HorizonPalette.textSecondary)),
                if (p.description != null)
                  Text(p.description!, maxLines: 2, overflow: TextOverflow.ellipsis,
                      style: const TextStyle(fontSize: 11, color: HorizonPalette.textSecondary)),
              ]),
            ),
          ]),
          const SizedBox(height: 8),
          Wrap(spacing: 6, runSpacing: 4, children: <Widget>[
            if (p.version != null) _Tag(ltr(p.version!)),
            if (p.source != null) _Tag(ltr(p.source!)),
            if (p.compatibility != null) _Tag(p.compatibility!),
            if (p.downloads != null) _Tag('${s.t('downloads')} ${ltr(_short(p.downloads!))}'),
            if (!p.installable) _Tag(s.t('notInstallable')),
            ...p.loaders.take(2).map((String l) => _Tag(ltr(l))),
          ]),
          const SizedBox(height: 10),
          Row(children: <Widget>[
            Expanded(
              child: _OutlineDetails(s: s, p: p),
            ),
            const SizedBox(width: 8),
            PremiumPressable(
              onTap: p.installable
                  ? () => ActionRunner(ref, context).run(HorizonAction.Install,
                      args: <String, Object?>{'index': p.index, 'kind': kind.name})
                  : () {},
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 9),
                decoration: BoxDecoration(
                  gradient: p.installable
                      ? HorizonPalette.primaryGradient
                      : const LinearGradient(colors: <Color>[HorizonPalette.border, HorizonPalette.border]),
                  borderRadius: BorderRadius.circular(HorizonRadii.pill),
                ),
                child: Text(p.installable ? s.t('install') : s.t('notInstallable'),
                    style: const TextStyle(fontSize: 12, color: Colors.white, fontWeight: FontWeight.w700)),
              ),
            ),
          ]),
        ]),
      );

  static String _short(int n) => n >= 1000000
      ? '${(n / 1000000).toStringAsFixed(1)}M'
      : n >= 1000 ? '${(n / 1000).toStringAsFixed(1)}K' : '$n';
}

class _OutlineDetails extends StatelessWidget {
  const _OutlineDetails({required this.s, required this.p});
  final HorizonStrings s;
  final CatalogProject p;

  @override
  Widget build(BuildContext context) => HorizonCard(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 9),
        onTap: () => showProjectDetails(context, s, p),
        child: adaptiveIconLabel(
          text: s.t('details'),
          icon: Icons.info_outline_rounded,
          iconSize: 16,
          iconColor: HorizonPalette.accent,
          style: const TextStyle(fontSize: 12),
        ),
      );
}

class _Icon extends StatelessWidget {
  const _Icon({this.url});
  final String? url;

  @override
  Widget build(BuildContext context) => ClipRRect(
        borderRadius: BorderRadius.circular(12),
        child: url == null || url!.isEmpty
            ? Container(
                width: 56, height: 56,
                color: HorizonPalette.secondary.withValues(alpha: 0.18),
                child: const Icon(Icons.image_outlined, color: HorizonPalette.accent),
              )
            : Image.network(
                url!,
                width: 56, height: 56, fit: BoxFit.cover,
                errorBuilder: (BuildContext _, Object __, StackTrace? ___) => Container(
                  width: 56, height: 56,
                  color: HorizonPalette.secondary.withValues(alpha: 0.18),
                  child: const Icon(Icons.image_outlined, color: HorizonPalette.accent),
                ),
              ),
      );
}

class _Tag extends StatelessWidget {
  const _Tag(this.label);
  final String label;

  @override
  Widget build(BuildContext context) => Container(
        margin: const EdgeInsets.only(top: 2),
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
        child: adaptiveIconLabel(
          text: label,
          icon: icon,
          iconSize: 17,
          iconColor: HorizonPalette.accent,
          style: const TextStyle(fontSize: 12),
        ),
      );
}
