import 'dart:math' as math;

import 'package:flutter/material.dart';

import 'horizon_theme.dart';

/// CRAFTIX Minecraft-inspired progress strip.
///
/// The runner position is driven ONLY by real download progress from
/// `official_install::install_with_progress` (via JNI). No timer, no invented
/// percentage and no repeating animation loop: `AnimatedPositioned` animates
/// only when a new real sample arrives. With reduced motion enabled the runner
/// is replaced by a static, fully readable progress indicator.
class CraftixRunnerProgress extends StatelessWidget {
  const CraftixRunnerProgress({
    super.key,
    required this.progress,
    required this.stage,
    required this.doneBytes,
    required this.totalBytes,
    required this.speedBytesPerSecond,
    required this.etaSeconds,
    required this.milestone,
    required this.label,
  });

  /// null = total size unknown -> indeterminate, never a fake percentage.
  final double? progress;
  final String? stage;
  final int doneBytes;
  final int totalBytes;
  final int? speedBytesPerSecond;
  final int? etaSeconds;
  final int milestone;
  final String label;

  @override
  Widget build(BuildContext context) {
    final bool reduced = MediaQuery.of(context).disableAnimations;
    final double? fraction = progress;
    final String numbers = _numbers();
    return Container(
      padding: const EdgeInsets.fromLTRB(16, 14, 16, 12),
      decoration: BoxDecoration(
        color: HorizonPalette.card,
        borderRadius: BorderRadius.circular(HorizonRadii.card),
        border: Border.all(color: HorizonPalette.border.withValues(alpha: 0.8)),
        boxShadow: <BoxShadow>[
          BoxShadow(color: HorizonPalette.primary.withValues(alpha: 0.16), blurRadius: 18),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisSize: MainAxisSize.min,
        children: <Widget>[
          Row(
            children: <Widget>[
              Expanded(
                child: Text(
                  label,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w800),
                ),
              ),
              if (fraction != null)
                Text(
                  '${(fraction * 100).round()}%',
                  style: const TextStyle(
                    fontSize: 20,
                    fontWeight: FontWeight.w900,
                    color: HorizonPalette.accent,
                  ),
                ),
            ],
          ),
          const SizedBox(height: 4),
          if (stage != null && stage!.isNotEmpty)
            Text(
              stage!,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(fontSize: 11, color: HorizonPalette.textSecondary),
            ),
          const SizedBox(height: 10),
          if (reduced)
            _StaticProgress(fraction: fraction)
          else
            _RunnerTrack(fraction: fraction, milestone: milestone),
          const SizedBox(height: 8),
          Text(
            numbers,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(fontSize: 10.5, color: HorizonPalette.textSecondary),
          ),
        ],
      ),
    );
  }

  String _numbers() {
    final String done = _bytes(doneBytes);
    final String total = totalBytes > 0 ? _bytes(totalBytes) : '—';
    final StringBuffer buffer = StringBuffer('$done / $total');
    if (speedBytesPerSecond != null && speedBytesPerSecond! > 0) {
      buffer.write('  •  ${_bytes(speedBytesPerSecond!)}/s');
    }
    if (etaSeconds != null && etaSeconds! > 0) {
      buffer.write('  •  ~${_duration(etaSeconds!)}');
    }
    return buffer.toString();
  }

  static String _bytes(int value) {
    if (value >= 1 << 30) return '${(value / (1 << 30)).toStringAsFixed(2)} GB';
    if (value >= 1 << 20) return '${(value / (1 << 20)).toStringAsFixed(1)} MB';
    if (value >= 1 << 10) return '${(value / (1 << 10)).toStringAsFixed(0)} KB';
    return '$value B';
  }

  static String _duration(int seconds) {
    if (seconds >= 60) return '${seconds ~/ 60}m ${seconds % 60}s';
    return '${seconds}s';
  }
}

class _StaticProgress extends StatelessWidget {
  const _StaticProgress({required this.fraction});
  final double? fraction;

  @override
  Widget build(BuildContext context) => ClipRRect(
        borderRadius: const BorderRadius.all(Radius.circular(8)),
        child: LinearProgressIndicator(value: fraction, minHeight: 8),
      );
}

class _RunnerTrack extends StatelessWidget {
  const _RunnerTrack({required this.fraction, required this.milestone});
  final double? fraction;
  final int milestone;

  @override
  Widget build(BuildContext context) => LayoutBuilder(
        builder: (BuildContext context, BoxConstraints constraints) {
          final double width = constraints.maxWidth;
          final double usable = math.max(0.0, width - 28);
          final double? target = fraction == null ? null : usable * fraction!.clamp(0.0, 1.0);
          return SizedBox(
            height: 44,
            child: Stack(
              clipBehavior: Clip.none,
              children: <Widget>[
                // Blocky ground track.
                Positioned.fill(
                  child: CustomPaint(painter: _TrackPainter(milestone: milestone)),
                ),
                // Real progress line (indeterminate when total is unknown).
                Positioned(
                  left: 0,
                  right: 0,
                  bottom: 4,
                  child: ClipRRect(
                    borderRadius: const BorderRadius.all(Radius.circular(6)),
                    child: fraction == null
                        ? const LinearProgressIndicator(minHeight: 6)
                        : LinearProgressIndicator(value: fraction, minHeight: 6),
                  ),
                ),
                // Pickups unlocked only by real milestone thresholds.
                for (int i = 1; i <= 4; i++)
                  if (milestone >= i)
                    Positioned(
                      left: usable * (i / 4.0) - 4,
                      bottom: 12,
                      child: _Pickup(finale: i == 4),
                    ),
                if (target != null)
                  AnimatedPositioned(
                    duration: const Duration(milliseconds: 420),
                    curve: Curves.easeOutCubic,
                    left: target,
                    bottom: 10,
                    child: const _Runner(),
                  ),
                if (milestone >= 4)
                  const Positioned(
                    right: 0,
                    bottom: 14,
                    child: _Pickup(finale: true),
                  ),
              ],
            ),
          );
        },
      );
}

class _TrackPainter extends CustomPainter {
  const _TrackPainter({required this.milestone});
  final int milestone;

  @override
  void paint(Canvas canvas, Size size) {
    final Paint grass = Paint()..color = const Color(0xFF2FBF71);
    final Paint dirt = Paint()..color = const Color(0xFF6B4423);
    final double ground = size.height - 10;
    canvas.drawRect(Rect.fromLTWH(0, ground + 4, size.width, 6), dirt);
    for (double x = 0; x < size.width; x += 12) {
      canvas.drawRect(Rect.fromLTWH(x, ground, 10, 5), grass);
    }
  }

  @override
  bool shouldRepaint(_TrackPainter oldDelegate) => oldDelegate.milestone != milestone;
}

class _Runner extends StatelessWidget {
  const _Runner();

  @override
  Widget build(BuildContext context) => SizedBox(
        width: 26,
        height: 34,
        child: CustomPaint(painter: _RunnerPainter()),
      );
}

class _RunnerPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final Paint skin = Paint()..color = const Color(0xFFD89B6C);
    final Paint shirt = Paint()..color = const Color(0xFF22D3EE);
    final Paint trousers = Paint()..color = const Color(0xFF315BFF);
    final Paint hair = Paint()..color = const Color(0xFF3B2416);
    final double w = size.width;
    // Voxel character: head, body, two legs, one arm carrying a block.
    canvas.drawRect(Rect.fromLTWH(w * 0.22, 0, w * 0.56, 11), hair);
    canvas.drawRect(Rect.fromLTWH(w * 0.22, 4, w * 0.56, 8), skin);
    canvas.drawRect(Rect.fromLTWH(w * 0.22, 13, w * 0.56, 11), shirt);
    canvas.drawRect(Rect.fromLTWH(w * 0.24, 24, w * 0.22, 9), trousers);
    canvas.drawRect(Rect.fromLTWH(w * 0.54, 24, w * 0.22, 9), trousers);
    final Paint block = Paint()..color = const Color(0xFF8B35F5);
    canvas.drawRect(Rect.fromLTWH(w * 0.78, 12, 8, 8), block);
  }

  @override
  bool shouldRepaint(_RunnerPainter oldDelegate) => false;
}

class _Pickup extends StatelessWidget {
  const _Pickup({required this.finale});
  final bool finale;

  @override
  Widget build(BuildContext context) => AnimatedScale(
        scale: finale ? 1.15 : 1.0,
        duration: const Duration(milliseconds: 260),
        child: Container(
          width: 12,
          height: 12,
          decoration: BoxDecoration(
            color: finale ? HorizonPalette.success : HorizonPalette.accent,
            borderRadius: const BorderRadius.all(Radius.circular(2)),
            boxShadow: <BoxShadow>[
              BoxShadow(
                color: (finale ? HorizonPalette.success : HorizonPalette.accent)
                    .withValues(alpha: 0.7),
                blurRadius: 10,
              ),
            ],
          ),
        ),
      );
}
