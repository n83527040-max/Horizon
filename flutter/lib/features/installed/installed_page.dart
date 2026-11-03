import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/bridge/actions.dart';
import '../../core/bridge/models.dart';
import '../../core/state/providers.dart';
import '../../core/theme/horizon_strings.dart';
import '../../core/theme/horizon_theme.dart';
import '../../core/widgets/horizon_widgets.dart';
import '../dialogs/dialogs.dart';

class InstalledPage extends ConsumerWidget {
  const InstalledPage({super.key});

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
            SectionTitle(s.t('installed'), subtitle: '${d.totalInstalled} items • enable, import or remove content'),
            ...ContentKind.values.map((ContentKind kind) => Padding(
                  padding: const EdgeInsets.only(bottom: 10),
                  child: HorizonCard(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: <Widget>[
                        Row(
                          children: <Widget>[
                            Icon(_icon(kind), color: HorizonPalette.primary, size: 20),
                            const SizedBox(width: 8),
                            Expanded(
                              child: Text(_label(kind, s), style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w700)),
                            ),
                            Text('${d.count(kind)}', style: const TextStyle(fontSize: 13, color: HorizonPalette.textSecondary)),
                          ],
                        ),
                        ...d.installed.where((InstalledItem i) => i.kind == kind).map((InstalledItem item) => Padding(
                              padding: const EdgeInsets.only(top: 10),
                              child: Row(
                                children: <Widget>[
                                  Expanded(
                                    child: Column(
                                      crossAxisAlignment: CrossAxisAlignment.start,
                                      children: <Widget>[
                                        Text(item.title, maxLines: 1, overflow: TextOverflow.ellipsis,
                                            style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w600)),
                                        Text(item.file, maxLines: 1, overflow: TextOverflow.ellipsis,
                                            style: const TextStyle(fontSize: 10, color: HorizonPalette.textSecondary)),
                                      ],
                                    ),
                                  ),
                                  IconButton(
                                    tooltip: item.enabled == true ? s.t('disable') : s.t('enable'),
                                    onPressed: () => ActionRunner(ref, context).run(
                                      HorizonAction.ToggleLocal,
                                      args: <String, Object?>{'index': item.index},
                                    ),
                                    icon: Icon(
                                      item.enabled == true ? Icons.toggle_on_rounded : Icons.toggle_off_rounded,
                                      color: item.enabled == true ? HorizonPalette.success : HorizonPalette.textSecondary,
                                    ),
                                  ),
                                  IconButton(
                                    tooltip: s.t('remove'),
                                    onPressed: () => ActionRunner(ref, context).run(
                                      HorizonAction.DeleteLocal,
                                      args: <String, Object?>{'index': item.index},
                                    ),
                                    icon: const Icon(Icons.delete_outline_rounded, size: 20, color: HorizonPalette.danger),
                                  ),
                                ],
                              ),
                            )),
                      ],
                    ),
                  ),
                )),
            const SizedBox(height: 12),
            if (d.totalInstalled == 0)
              HorizonCard(
                child: EmptyState(
                  title: s.t('noContent'),
                  hint: s.t('noContentHint'),
                  icon: Icons.inventory_2_outlined,
                  action: GradientButton(
                    label: s.t('importContent'),
                    icon: Icons.add_rounded,
                    onTap: () => showImportDialog(context, ref, ContentKind.Mods),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }

  static IconData _icon(ContentKind kind) {
    switch (kind) {
      case ContentKind.Mods: return Icons.extension_rounded;
      case ContentKind.Packs: return Icons.layers_rounded;
      case ContentKind.Shaders: return Icons.auto_awesome_rounded;
    }
  }

  static String _label(ContentKind kind, HorizonStrings s) {
    switch (kind) {
      case ContentKind.Mods: return s.t('mods');
      case ContentKind.Packs: return 'Resource Packs';
      case ContentKind.Shaders: return s.t('shaders');
    }
  }
}
