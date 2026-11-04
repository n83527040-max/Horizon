import 'package:flutter/material.dart';

import 'horizon_theme.dart';

/// The download card displays only the installer's real progress and transfer
/// statistics. The voxel scene is decorative; it does not indicate progress.
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

  final double? progress;
  final String? stage;
  final int doneBytes;
  final int totalBytes;
  final int? speedBytesPerSecond;
  final int? etaSeconds;
  // Retained for bridge/API compatibility; artwork is not a progress indicator.
  final int milestone;
  final String label;

  @override
  Widget build(BuildContext context) {
    final double? fraction = progress?.clamp(0.0, 1.0).toDouble();
    final String amounts = '${_bytes(doneBytes)} / ${totalBytes > 0 ? _bytes(totalBytes) : '—'}';
    return Container(
      padding: const EdgeInsets.fromLTRB(14, 12, 14, 11),
      decoration: BoxDecoration(
        gradient: const LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: <Color>[Color(0xFF21103B), Color(0xFF0D102A), Color(0xFF11102D)],
        ),
        borderRadius: BorderRadius.circular(HorizonRadii.card),
        border: Border.all(color: const Color(0xFFB53CFF), width: 1),
        boxShadow: <BoxShadow>[
          BoxShadow(color: const Color(0xFF9A28F5).withValues(alpha: 0.3), blurRadius: 16),
          BoxShadow(color: const Color(0xFF9A28F5).withValues(alpha: 0.16), blurRadius: 4),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Row(
            children: <Widget>[
              const _DownloadIcon(),
              const SizedBox(width: 10),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: <Widget>[
                    Text(label, maxLines: 1, overflow: TextOverflow.ellipsis,
                        style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w800, color: Colors.white)),
                    if (stage != null && stage!.isNotEmpty)
                      Text(stage!, maxLines: 1, overflow: TextOverflow.ellipsis,
                          style: const TextStyle(fontSize: 10.5, color: Color(0xFFC8B9E9))),
                  ],
                ),
              ),
              if (fraction != null) ...<Widget>[
                const SizedBox(width: 8),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 3),
                  decoration: BoxDecoration(
                    color: const Color(0xFF2B0C53),
                    borderRadius: BorderRadius.circular(30),
                    border: Border.all(color: const Color(0xFFE080FF)),
                    boxShadow: <BoxShadow>[
                      BoxShadow(color: const Color(0xFFBD28FF).withValues(alpha: 0.65), blurRadius: 9),
                    ],
                  ),
                  child: Text('${(fraction * 100).round()}%',
                      style: const TextStyle(color: Colors.white, fontSize: 16, fontWeight: FontWeight.w900)),
                ),
              ],
            ],
          ),
          const SizedBox(height: 4),
          Expanded(
            child: ExcludeSemantics(
              child: Image.asset('assets/download_scene.png', fit: BoxFit.contain,
                  errorBuilder: (_, __, ___) => const SizedBox.shrink()),
            ),
          ),
          const SizedBox(height: 4),
          _NeonProgress(fraction: fraction),
          const SizedBox(height: 8),
          Directionality(
            textDirection: TextDirection.ltr,
            child: Row(
              children: <Widget>[
                Expanded(flex: 5, child: _Stat(Icons.download_rounded, amounts)),
                if (speedBytesPerSecond != null && speedBytesPerSecond! > 0)
                  Expanded(flex: 3, child: _Stat(Icons.speed_rounded, '${_bytes(speedBytesPerSecond!)}/s')),
                if (etaSeconds != null && etaSeconds! > 0)
                  Expanded(flex: 2, child: _Stat(Icons.access_time_rounded, '~ ${_duration(etaSeconds!)}')),
              ],
            ),
          ),
        ],
      ),
    );
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

class _DownloadIcon extends StatelessWidget {
  const _DownloadIcon();

  @override
  Widget build(BuildContext context) => Container(
        decoration: BoxDecoration(boxShadow: <BoxShadow>[
          BoxShadow(color: const Color(0xFFBE36FF).withValues(alpha: 0.55), blurRadius: 16),
        ]),
        child: const Icon(Icons.download_rounded, size: 28, color: Color(0xFFE47CFF)),
      );
}

class _Stat extends StatelessWidget {
  const _Stat(this.icon, this.text);
  final IconData icon;
  final String text;

  @override
  Widget build(BuildContext context) => Row(
        children: <Widget>[
          Icon(icon, size: 15, color: const Color(0xFFD57AFF)),
          const SizedBox(width: 4),
          Flexible(child: Text(text, maxLines: 1, overflow: TextOverflow.ellipsis,
              style: const TextStyle(fontSize: 10, color: Color(0xFFF4ECFF)))),
        ],
      );
}

class _NeonProgress extends StatelessWidget {
  const _NeonProgress({required this.fraction});
  final double? fraction;

  @override
  Widget build(BuildContext context) => Container(
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(20),
          border: Border.all(color: const Color(0xFF7740AE)),
          boxShadow: <BoxShadow>[
            BoxShadow(color: const Color(0xFF9A28F5).withValues(alpha: 0.35), blurRadius: 7),
          ],
        ),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(20),
          child: LinearProgressIndicator(
            value: fraction,
            minHeight: 8,
            backgroundColor: const Color(0xFF0A0820),
            valueColor: const AlwaysStoppedAnimation<Color>(Color(0xFFD75BFF)),
          ),
        ),
      );
}
