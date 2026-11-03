import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../bridge/actions.dart';
import '../bridge/launcher_bridge.dart';
import '../bridge/models.dart';
import '../theme/horizon_strings.dart';
import '../theme/horizon_theme.dart';

final bridgeProvider = Provider<LauncherBridge>((Ref ref) {
  return MethodChannelLauncherBridge();
});

/// Polls the real native snapshot. Errors are surfaced as AsyncError and the UI
/// offers a real retry; no cached/demo snapshot is substituted.
final snapshotProvider = StreamProvider<LauncherSnapshot>((Ref ref) async* {
  final LauncherBridge bridge = ref.watch(bridgeProvider);
  while (true) {
    try {
      yield await bridge.snapshot();
    } on Object catch (error, stack) {
      yield* Stream<LauncherSnapshot>.error(error, stack);
      return;
    }
    await Future<void>.delayed(const Duration(seconds: 4));
  }
});

/// Real asynchronous results from the native host (SAF import, game-launch
/// failure, Microsoft sign-in code). No synthetic event is ever produced here.
final nativeEventsProvider = StreamProvider<Map<String, Object?>>((Ref ref) {
  return ref.watch(bridgeProvider).events;
});

final navIndexProvider = StateProvider<int>((Ref ref) => 0);

final stringsProvider = Provider<HorizonStrings>((Ref ref) {
  final AsyncValue<LauncherSnapshot> state = ref.watch(snapshotProvider);
  return HorizonStrings.of(
    state.valueOrNull?.setting(SettingKeys.language) as String?,
  );
});

final themeModeProvider = Provider<ThemeMode>((Ref ref) {
  final AsyncValue<LauncherSnapshot> state = ref.watch(snapshotProvider);
  switch (state.valueOrNull?.setting(SettingKeys.theme)) {
    case 'light':
      return ThemeMode.light;
    case 'dark':
      return ThemeMode.dark;
    default:
      return ThemeMode.system;
  }
});

final localeProvider = Provider<Locale>((Ref ref) {
  final AsyncValue<LauncherSnapshot> state = ref.watch(snapshotProvider);
  return state.valueOrNull?.setting(SettingKeys.language) == 'ar'
      ? const Locale('ar')
      : const Locale('en');
});

/// A single place for press feedback. Every interactive element uses this so
/// success/failure/loading from the native host is always visible to the player.
class ActionRunner {
  const ActionRunner(this.ref, this.context);

  final WidgetRef ref;
  final BuildContext context;

  Future<void> run(
    HorizonAction action, {
    Map<String, Object?> args = const <String, Object?>{},
  }) async {
    try {
      final LauncherBridge bridge = ref.read(bridgeProvider);
      final BridgeResult result = await bridge.dispatch(action, args: args);
      if (!context.mounted) return;
      final HorizonStrings s = ref.read(stringsProvider);
      final String message = s.native(result.message.isEmpty ? action.name : result.message);
      _feedback(
        context,
        message: message,
        ok: result.ok,
        duration: result.pending || result.needsActivity
            ? const Duration(seconds: 4)
            : const Duration(seconds: 2),
        pending: result.pending || result.needsActivity,
      );
      if (result.changed || result.pending) ref.invalidate(snapshotProvider);
    } on Object catch (error) {
      _feedback(context, message: '$error', ok: false);
    }
  }

  /// Floating SnackBar above the fixed bottom navigation (never covers it).
  void _feedback(
    BuildContext context, {
    required String message,
    required bool ok,
    Duration duration = const Duration(seconds: 2),
    bool pending = false,
  }) {
    if (!context.mounted) return;
    final HorizonStrings s = ref.read(stringsProvider);
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(SnackBar(
        content: Text(message),
        behavior: SnackBarBehavior.floating,
        margin: const EdgeInsets.fromLTRB(16, 0, 16, 82),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(HorizonRadii.control)),
        backgroundColor: ok ? HorizonPalette.card : HorizonPalette.danger,
        duration: duration,
        action: pending
            ? SnackBarAction(label: s.t('cancel'), onPressed: () => cancel())
            : null,
      ));
  }

  /// Requests real cancellation from Rust (`nativeCancel`). Downloads that the
  /// upstream downloader cannot abort are reported honestly by the snapshot.
  Future<void> cancel() async {
    try {
      final BridgeResult result = await ref.read(bridgeProvider).cancel();
      ref.invalidate(snapshotProvider);
      if (!context.mounted) return;
      _feedback(context,
          message: ref.read(stringsProvider).native(
              result.message.isEmpty ? 'Cancellation requested' : result.message),
          ok: result.ok);
    } on Object catch (error) {
      if (!context.mounted) return;
      _feedback(context, message: '$error', ok: false);
    }
  }
}
