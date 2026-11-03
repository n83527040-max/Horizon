import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../bridge/actions.dart';
import '../bridge/launcher_bridge.dart';
import '../bridge/models.dart';
import '../theme/horizon_strings.dart';
import '../theme/horizon_theme.dart';

final bridgeProvider = Provider<LauncherBridge>((Ref ref) {
  return const MethodChannelLauncherBridge();
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
/// success/failure from the native host is always visible to the player.
class ActionRunner {
  const ActionRunner(this.ref, this.context);

  final WidgetRef ref;
  final BuildContext context;

  Future<void> run(
    HorizonAction action, {
    Map<String, Object?> args = const <String, Object?>{},
  }) async {
    try {
      final BridgeResult result = await ref.read(bridgeProvider).dispatch(action, args: args);
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(result.message.isEmpty ? action.name : result.message),
          backgroundColor: result.ok ? HorizonPalette.card : HorizonPalette.danger,
          duration: const Duration(seconds: 2),
        ),
      );
      if (result.changed) ref.invalidate(snapshotProvider);
    } on Object catch (error) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('$error'), backgroundColor: HorizonPalette.danger),
      );
    }
  }
}
