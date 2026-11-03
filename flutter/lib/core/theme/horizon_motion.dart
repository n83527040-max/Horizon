import 'package:flutter/material.dart';

import 'horizon_theme.dart';

/// Premium motion that stays cheap: short, finite, GPU-friendly opacity/offset
/// transitions. No repeating shimmer loops (battery) and no page rebuilds
/// (IndexedStack keeps page state). Every controller is disposed with its widget.

/// Forces a left-to-right embedding for Latin/numeric fragments inside RTL
/// Arabic text. Fixes `MB / 3.7 GB 180` and `items • ... 0` in the screenshots.
String ltr(String value) => '\u202A$value\u202C';

/// A short fade + slide entrance. `index` staggers siblings without timers.
class StaggeredEntrance extends StatefulWidget {
  const StaggeredEntrance({super.key, this.index = 0, required this.child});

  final int index;
  final Widget child;

  @override
  State<StaggeredEntrance> createState() => _StaggeredEntranceState();
}

class _StaggeredEntranceState extends State<StaggeredEntrance>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 300),
  );

  @override
  void initState() {
    super.initState();
    _controller.forward();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final double begin = (widget.index * 0.06).clamp(0.0, 0.48);
    final Animation<double> fade = CurvedAnimation(
      parent: _controller,
      curve: Interval(begin, 1.0, curve: Curves.easeOut),
    );
    final Animation<Offset> slide = Tween<Offset>(
      begin: const Offset(0, 0.045),
      end: Offset.zero,
    ).animate(CurvedAnimation(
      parent: _controller,
      curve: Interval(begin, 1.0, curve: Curves.easeOutCubic),
    ));
    return FadeTransition(
      opacity: fade,
      child: SlideTransition(position: slide, child: widget.child),
    );
  }
}

/// A real indeterminate loading placeholder (not fake data).
class HorizonSkeleton extends StatelessWidget {
  const HorizonSkeleton({super.key, this.height = 84, this.label});

  final double height;
  final String? label;

  @override
  Widget build(BuildContext context) => HorizonCard(
        child: SizedBox(
          height: height,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            mainAxisAlignment: MainAxisAlignment.center,
            children: <Widget>[
              if (label != null)
                Padding(
                  padding: const EdgeInsets.only(bottom: 10),
                  child: Text(label!, style: const TextStyle(fontSize: 11.5)),
                ),
              const ClipRRect(
                borderRadius: BorderRadius.all(Radius.circular(8)),
                child: LinearProgressIndicator(minHeight: 4),
              ),
              const SizedBox(height: 12),
              Container(
                height: 10,
                decoration: BoxDecoration(
                  color: HorizonPalette.card.withValues(alpha: 0.9),
                  borderRadius: BorderRadius.circular(5),
                ),
              ),
              const SizedBox(height: 6),
              FractionallySizedBox(
                widthFactor: 0.65,
                child: Container(
                  height: 9,
                  decoration: BoxDecoration(
                    color: HorizonPalette.card.withValues(alpha: 0.9),
                    borderRadius: BorderRadius.circular(5),
                  ),
                ),
              ),
            ],
          ),
        ),
      );
}

/// Animated, real progress from the native download worker (0..1).
class AnimatedProgress extends StatelessWidget {
  const AnimatedProgress({super.key, required this.value, this.label});

  final double value;
  final String? label;

  @override
  Widget build(BuildContext context) => TweenAnimationBuilder<double>(
        tween: Tween<double>(begin: 0, end: value.clamp(0.0, 1.0)),
        duration: const Duration(milliseconds: 260),
        curve: Curves.easeOut,
        builder: (BuildContext context, double animated, Widget? _) => Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            if (label != null)
              Padding(
                padding: const EdgeInsets.only(bottom: 6),
                child: Text(label!, style: const TextStyle(fontSize: 11)),
              ),
            ClipRRect(
              borderRadius: const BorderRadius.all(Radius.circular(8)),
              child: LinearProgressIndicator(value: animated, minHeight: 6),
            ),
          ],
        ),
      );
}

/// Animated scale/opacity for dialogs and bottom sheets.
class AnimatedSheet extends StatefulWidget {
  const AnimatedSheet({super.key, required this.child});

  final Widget child;

  @override
  State<AnimatedSheet> createState() => _AnimatedSheetState();
}

class _AnimatedSheetState extends State<AnimatedSheet>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 240),
  );

  @override
  void initState() {
    super.initState();
    _controller.forward();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final Animation<double> curved = CurvedAnimation(
      parent: _controller,
      curve: Curves.easeOutCubic,
    );
    return FadeTransition(
      opacity: curved,
      child: ScaleTransition(
        scale: Tween<double>(begin: 0.96, end: 1.0).animate(curved),
        alignment: Alignment.bottomCenter,
        child: widget.child,
      ),
    );
  }
}

/// Fast press feedback with a light neon glow; no state loss, no rebuild of data.
class PremiumPressable extends StatefulWidget {
  const PremiumPressable({super.key, required this.onTap, required this.child});

  final VoidCallback onTap;
  final Widget child;

  @override
  State<PremiumPressable> createState() => _PremiumPressableState();
}

class _PremiumPressableState extends State<PremiumPressable> {
  bool _down = false;

  @override
  Widget build(BuildContext context) => GestureDetector(
        onTapDown: (_) => setState(() => _down = true),
        onTapUp: (_) => setState(() => _down = false),
        onTapCancel: () => setState(() => _down = false),
        onTap: widget.onTap,
        child: AnimatedScale(
          scale: _down ? 0.965 : 1.0,
          duration: const Duration(milliseconds: 110),
          curve: Curves.easeOut,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 140),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(HorizonRadii.card),
              boxShadow: _down
                  ? <BoxShadow>[
                      BoxShadow(
                        color: HorizonPalette.primary.withValues(alpha: 0.35),
                        blurRadius: 18,
                      ),
                    ]
                  : const <BoxShadow>[],
            ),
            child: widget.child,
          ),
        ),
      );
}


/// A one-shot fade/slide on page change. `key` changes with the nav index but
/// the child (IndexedStack) is created from the same const widget list, so page
/// state is preserved while the whole stack transitions smoothly.
class AnimatedPageFade extends StatefulWidget {
  const AnimatedPageFade({super.key, required this.index, required this.child});

  /// Changing this replays the transition but keeps the SAME element/subtree.
  final int index;
  final Widget child;

  @override
  State<AnimatedPageFade> createState() => _AnimatedPageFadeState();
}

class _AnimatedPageFadeState extends State<AnimatedPageFade>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 220),
  )..forward();

  @override
  void didUpdateWidget(AnimatedPageFade oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.index != widget.index) {
      _controller.forward(from: 0);
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => FadeTransition(
        opacity: CurvedAnimation(parent: _controller, curve: Curves.easeOut),
        child: SlideTransition(
          position: Tween<Offset>(
            begin: const Offset(0, 0.012),
            end: Offset.zero,
          ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeOut)),
          child: widget.child,
        ),
      );
}
