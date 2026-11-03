import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../bridge/models.dart';
import '../theme/horizon_strings.dart';
import '../theme/horizon_theme.dart';

/// A label row that only uses [Flexible] when its incoming width is finite.
/// Rows inside `Wrap`, `EmptyState` or shrink-wrapped columns receive unbounded
/// width; a flex child there throws "RenderFlex children have non-zero flex but
/// incoming width constraints are unbounded". This helper shrink-wraps in that
/// case and ellipsizes when width is bounded, on every screen size.
Widget adaptiveIconLabel({
  required String text,
  required TextStyle style,
  IconData? icon,
  Color? iconColor,
  double iconSize = 18,
  double gap = 6,
  int maxLines = 1,
  MainAxisAlignment alignment = MainAxisAlignment.center,
  Widget? label,
}) {
  return LayoutBuilder(
    builder: (BuildContext context, BoxConstraints constraints) {
      final Widget content = label ??
          Text(text, maxLines: maxLines, overflow: TextOverflow.ellipsis, style: style);
      final bool bounded = constraints.hasBoundedWidth;
      return Row(
        mainAxisSize: bounded ? MainAxisSize.max : MainAxisSize.min,
        mainAxisAlignment: alignment,
        children: <Widget>[
          if (icon != null) Icon(icon, size: iconSize, color: iconColor),
          if (icon != null) SizedBox(width: gap),
          if (bounded)
            Flexible(child: content)
          else
            content,
        ],
      );
    },
  );
}

/// Rounded, subtly glowing card used by every page.
class HorizonCard extends StatelessWidget {
  const HorizonCard({
    super.key,
    required this.child,
    this.padding = const EdgeInsets.all(HorizonSpacing.card),
    this.glow = false,
    this.onTap,
    this.color,
  });

  final Widget child;
  final EdgeInsets padding;
  final bool glow;
  final VoidCallback? onTap;
  final Color? color;

  @override
  Widget build(BuildContext context) {
    final bool dark = Theme.of(context).brightness == Brightness.dark;
    return DecoratedBox(
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(HorizonRadii.card),
        boxShadow: <BoxShadow>[
          if (glow)
            BoxShadow(
              color: HorizonPalette.primary.withValues(alpha: 0.22),
              blurRadius: 18,
              spreadRadius: 1,
            ),
        ],
      ),
      child: Material(
        color: color ?? (dark ? HorizonPalette.card : Colors.white),
        borderRadius: BorderRadius.circular(HorizonRadii.card),
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(HorizonRadii.card),
          child: Container(
            padding: padding,
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(HorizonRadii.card),
              border: Border.all(
                color: dark ? HorizonPalette.border : const Color(0xFFDCE1F2),
                width: 1,
              ),
            ),
            child: child,
          ),
        ),
      ),
    );
  }
}

/// Lightweight press animation: scales down on tap, never heavy repaints.
class HorizonPressable extends StatefulWidget {
  const HorizonPressable({super.key, required this.child, required this.onTap});

  final Widget child;
  final VoidCallback onTap;

  @override
  State<HorizonPressable> createState() => _HorizonPressableState();
}

class _HorizonPressableState extends State<HorizonPressable> {
  bool _down = false;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTapDown: (_) => setState(() => _down = true),
      onTapUp: (_) => setState(() => _down = false),
      onTapCancel: () => setState(() => _down = false),
      onTap: widget.onTap,
      child: AnimatedScale(
        scale: _down ? 0.97 : 1,
        duration: const Duration(milliseconds: 110),
        curve: Curves.easeOut,
        child: widget.child,
      ),
    );
  }
}

class GradientButton extends StatelessWidget {
  const GradientButton({
    super.key,
    required this.label,
    required this.onTap,
    this.icon,
    this.height = 54,
  });

  final String label;
  final VoidCallback onTap;
  final IconData? icon;
  final double height;

  @override
  Widget build(BuildContext context) {
    return HorizonPressable(
      onTap: onTap,
      child: Container(
        height: height,
        decoration: BoxDecoration(
          gradient: HorizonPalette.primaryGradient,
          borderRadius: BorderRadius.circular(HorizonRadii.control),
          boxShadow: <BoxShadow>[
            BoxShadow(
              color: HorizonPalette.primary.withValues(alpha: 0.35),
              blurRadius: 16,
              offset: const Offset(0, 6),
            ),
          ],
        ),
        alignment: Alignment.center,
        child: adaptiveIconLabel(
          text: label,
          icon: icon,
          iconColor: Colors.white,
          iconSize: 20,
          gap: 8,
          style: const TextStyle(
            color: Colors.white,
            fontSize: 17,
            fontWeight: FontWeight.w700,
            letterSpacing: 0.4,
          ),
        ),
      ),
    );
  }
}

class HorizonChip extends StatelessWidget {
  const HorizonChip({
    super.key,
    required this.label,
    this.active = false,
    this.icon,
    this.onTap,
  });

  final String label;
  final bool active;
  final IconData? icon;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final bool dark = Theme.of(context).brightness == Brightness.dark;
    return HorizonPressable(
      onTap: onTap ?? () {},
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
        decoration: BoxDecoration(
          gradient: active ? HorizonPalette.primaryGradient : null,
          color: active ? null : (dark ? HorizonPalette.card : Colors.white),
          borderRadius: BorderRadius.circular(HorizonRadii.pill),
          border: Border.all(color: active ? Colors.transparent : HorizonPalette.border),
        ),
        child: adaptiveIconLabel(
          text: label,
          icon: icon,
          iconColor: Colors.white,
          iconSize: 16,
          style: TextStyle(
            color: active ? Colors.white : Theme.of(context).colorScheme.onSurface,
            fontWeight: active ? FontWeight.w700 : FontWeight.w500,
            fontSize: 13,
          ),
        ),
      ),
    );
  }
}

class SectionTitle extends StatelessWidget {
  const SectionTitle(this.title, {super.key, this.subtitle, this.trailing});

  final String title;
  final String? subtitle;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 10, top: 4),
      child: Row(
        children: <Widget>[
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                Text(title, style: const TextStyle(fontSize: 20, fontWeight: FontWeight.w800)),
                if (subtitle != null)
                  Padding(
                    padding: const EdgeInsets.only(top: 2),
                    child: Text(
                      subtitle!,
                      style: TextStyle(fontSize: 12, color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.65)),
                    ),
                  ),
              ],
            ),
          ),
          if (trailing != null) trailing!,
        ],
      ),
    );
  }
}

class MetricTile extends StatelessWidget {
  const MetricTile({super.key, required this.label, required this.value, required this.icon});

  final String label;
  final String value;
  final IconData icon;

  @override
  Widget build(BuildContext context) {
    return adaptiveIconLabel(
      text: value,
      icon: icon,
      iconSize: 15,
      iconColor: HorizonPalette.accent,
      alignment: MainAxisAlignment.start,
      style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w700),
      label: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          Text(value, maxLines: 1, overflow: TextOverflow.ellipsis,
              style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w700)),
          Text(label, maxLines: 1, overflow: TextOverflow.ellipsis,
              style: const TextStyle(fontSize: 10, color: HorizonPalette.textSecondary)),
        ],
      ),
    );
  }
}

class EmptyState extends StatelessWidget {
  const EmptyState({super.key, required this.title, required this.hint, this.icon = Icons.inbox_outlined, this.action});

  final String title;
  final String hint;
  final IconData icon;
  final Widget? action;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            Icon(icon, size: 44, color: HorizonPalette.primary.withValues(alpha: 0.75)),
            const SizedBox(height: 12),
            Text(title, textAlign: TextAlign.center,
                style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w700)),
            const SizedBox(height: 6),
            Text(hint, textAlign: TextAlign.center,
                style: const TextStyle(fontSize: 12, color: HorizonPalette.textSecondary)),
            if (action != null) ...<Widget>[const SizedBox(height: 16), action!],
          ],
        ),
      ),
    );
  }
}

class AsyncSnapshotView extends StatelessWidget {
  const AsyncSnapshotView({
    super.key,
    required this.state,
    required this.builder,
    required this.strings,
    required this.onRetry,
  });

  final AsyncValue<LauncherSnapshot> state;
  final Widget Function(LauncherSnapshot snapshot) builder;
  final HorizonStrings strings;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return state.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (Object error, StackTrace _) => EmptyState(
        icon: Icons.error_outline,
        title: strings.t('error'),
        hint: '$error',
        action: FilledButton(onPressed: onRetry, child: Text(strings.t('retry'))),
      ),
      data: builder,
    );
  }
}
