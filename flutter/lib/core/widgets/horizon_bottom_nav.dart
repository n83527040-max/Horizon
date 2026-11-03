import 'package:flutter/material.dart';

import '../theme/horizon_theme.dart';

/// Fixed, full-width bottom navigation with a glowing active indicator.
/// It fills the Android navigation-bar safe area instead of covering content.
class HorizonBottomNav extends StatelessWidget {
  const HorizonBottomNav({
    super.key,
    required this.labels,
    required this.index,
    required this.onSelected,
  });

  final List<String> labels;
  final int index;
  final ValueChanged<int> onSelected;

  static const List<IconData> icons = <IconData>[
    Icons.home_rounded,
    Icons.extension_rounded,
    Icons.layers_rounded,
    Icons.inventory_2_rounded,
    Icons.settings_rounded,
  ];

  @override
  Widget build(BuildContext context) {
    final HorizonMetrics metrics = HorizonMetrics.of(context);
    final double bottom = metrics.padding.bottom;
    final bool dark = Theme.of(context).brightness == Brightness.dark;
    return DecoratedBox(
      decoration: BoxDecoration(
        color: (dark ? const Color(0xFF0C1330) : Colors.white).withValues(alpha: 0.94),
        border: Border(top: BorderSide(color: dark ? HorizonPalette.border : const Color(0xFFDCE1F2))),
        boxShadow: <BoxShadow>[
          BoxShadow(color: HorizonPalette.primary.withValues(alpha: 0.12), blurRadius: 20),
        ],
      ),
      child: Padding(
        padding: EdgeInsets.only(
          top: 8,
          bottom: bottom > 0 ? bottom : 10,
          left: 8,
          right: 8,
        ),
        child: Row(
          children: List<Widget>.generate(labels.length, (int i) {
            final bool active = i == index;
            return Expanded(
              child: Semantics(
                label: labels[i],
                selected: active,
                button: true,
                child: InkResponse(
                  onTap: () => onSelected(i),
                  radius: 32,
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: <Widget>[
                      AnimatedContainer(
                        duration: const Duration(milliseconds: 180),
                        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 3),
                        decoration: BoxDecoration(
                          color: active ? HorizonPalette.primary.withValues(alpha: 0.18) : Colors.transparent,
                          borderRadius: BorderRadius.circular(HorizonRadii.pill),
                        ),
                        child: Icon(
                          i < icons.length ? icons[i] : Icons.circle,
                          size: 21,
                          color: active ? HorizonPalette.primary : HorizonPalette.textSecondary,
                        ),
                      ),
                      const SizedBox(height: 3),
                      Text(
                        labels[i],
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: TextStyle(
                          fontSize: 10.5,
                          fontWeight: active ? FontWeight.w700 : FontWeight.w500,
                          color: active ? HorizonPalette.primary : HorizonPalette.textSecondary,
                        ),
                      ),
                      if (active)
                        Container(
                          margin: const EdgeInsets.only(top: 3),
                          width: 18,
                          height: 2,
                          decoration: BoxDecoration(
                            color: HorizonPalette.primary,
                            borderRadius: BorderRadius.circular(2),
                            boxShadow: <BoxShadow>[
                              BoxShadow(color: HorizonPalette.primary, blurRadius: 8),
                            ],
                          ),
                        ),
                    ],
                  ),
                ),
              ),
            );
          }),
        ),
      ),
    );
  }
}
