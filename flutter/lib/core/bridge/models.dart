import 'actions.dart';

/// Everything the UI displays comes from the native host. Every field is
/// nullable/empty when the engine does not expose a value: the UI hides it
/// instead of showing a demo number.
class LauncherSnapshot {
  const LauncherSnapshot({
    required this.appVersion,
    required this.gameVersion,
    this.profileName,
    this.accountStatus,
    this.avatarPath,
    this.graphicsApi,
    this.renderScalePercent,
    this.ramUsedMb,
    this.ramPeakMb,
    this.ramTotalMb,
    this.installed = const <InstalledItem>[],
    this.results = const <CatalogProject>[],
    this.status,
    this.settings = const <String, Object?>{},
    this.choices = const <String, List<String>>{},
    this.storageBytes,
    this.cacheBytes,
    this.contentBytes = const <String, int>{},
    this.engineBridgeReady = false,
    this.pending = false,
    this.loginCode,
    this.loginUri,
    this.loginMessage,
  });

  final String appVersion;
  final String gameVersion;
  final String? profileName;
  final String? accountStatus;
  final String? avatarPath;
  final String? graphicsApi;
  final int? renderScalePercent;
  final int? ramUsedMb;
  final int? ramPeakMb;
  final int? ramTotalMb;
  final List<InstalledItem> installed;
  final List<CatalogProject> results;
  final String? status;
  final Map<String, Object?> settings;
  final Map<String, List<String>> choices;
  final int? storageBytes;
  final int? cacheBytes;
  final Map<String, int> contentBytes;
  final bool engineBridgeReady;
  /// True while a real download, search or Microsoft sign-in is running.
  final bool pending;
  /// Microsoft device-code values, only present while sign-in is running.
  final String? loginCode;
  final String? loginUri;
  final String? loginMessage;

  int count(ContentKind kind) => installed.where((i) => i.kind == kind).length;
  int get totalInstalled => installed.length;

  Object? setting(String key) => settings[key];

  factory LauncherSnapshot.fromJson(Map<Object?, Object?> raw) {
    Map<Object?, Object?> map(Object? value) =>
        value is Map<Object?, Object?> ? value : const <Object?, Object?>{};
    List<Object?> list(Object? value) => value is List<Object?> ? value : const <Object?>[];
    int? integer(Object? value) => value is num ? value.toInt() : null;
    String? text(Object? value) => value is String && value.isNotEmpty ? value : null;

    return LauncherSnapshot(
      appVersion: text(raw['appVersion']) ?? '',
      gameVersion: text(raw['gameVersion']) ?? '',
      profileName: text(raw['profileName']),
      accountStatus: text(raw['accountStatus']),
      avatarPath: text(raw['avatarPath']),
      graphicsApi: text(raw['graphicsApi']),
      renderScalePercent: integer(raw['renderScalePercent']),
      ramUsedMb: integer(raw['ramUsedMb']),
      ramPeakMb: integer(raw['ramPeakMb']),
      ramTotalMb: integer(raw['ramTotalMb']),
      installed: list(raw['installed'])
          .whereType<Map<Object?, Object?>>()
          .map(InstalledItem.fromJson)
          .toList(growable: false),
      results: list(raw['results'])
          .whereType<Map<Object?, Object?>>()
          .map(CatalogProject.fromJson)
          .toList(growable: false),
      status: text(raw['status']),
      settings: Map<String, Object?>.from(map(raw['settings'])
          .map((Object? k, Object? v) => MapEntry<String, Object?>('$k', v))),
      choices: map(raw['choices']).map(
        (Object? k, Object? v) => MapEntry<String, List<String>>(
          '$k',
          list(v).whereType<String>().toList(growable: false),
        ),
      ),
      storageBytes: integer(raw['storageBytes']),
      cacheBytes: integer(raw['cacheBytes']),
      contentBytes: map(raw['contentBytes']).map(
        (Object? k, Object? v) => MapEntry<String, int>('$k', v is num ? v.toInt() : 0),
      ),
      engineBridgeReady: raw['engineBridgeReady'] == true,
      pending: raw['pending'] == true,
      loginCode: text(raw['loginCode']),
      loginUri: text(raw['loginUri']),
      loginMessage: text(raw['loginMessage']),
    );
  }
}

class InstalledItem {
  const InstalledItem({
    required this.file,
    required this.title,
    required this.kind,
    this.enabled,
    this.index = 0,
  });

  final String file;
  final String title;
  final ContentKind kind;
  /// null = the native host has not exposed enable/disable state yet.
  final bool? enabled;
  final int index;

  factory InstalledItem.fromJson(Map<Object?, Object?> raw) => InstalledItem(
        file: raw['file'] is String ? raw['file']! as String : '',
        title: raw['title'] is String ? raw['title']! as String : (raw['file'] ?? '').toString(),
        kind: ContentKind.fromFolder(raw['folder'] is String ? raw['folder']! as String : null) ??
            ContentKind.Mods,
        enabled: raw['enabled'] is bool ? raw['enabled']! as bool : null,
        index: raw['index'] is num ? (raw['index']! as num).toInt() : 0,
      );
}

class CatalogProject {
  const CatalogProject({
    required this.title,
    this.description,
    this.version,
    this.source,
    this.iconUrl,
    this.compatibility,
    required this.index,
  });

  final String title;
  final String? description;
  final String? version;
  final String? source;
  final String? iconUrl;
  final String? compatibility;
  final int index;

  factory CatalogProject.fromJson(Map<Object?, Object?> raw) => CatalogProject(
        title: raw['title'] is String ? raw['title']! as String : '',
        description: raw['description'] is String ? raw['description']! as String : null,
        version: raw['version'] is String ? raw['version']! as String : null,
        source: raw['source'] is String ? raw['source']! as String : null,
        iconUrl: raw['iconUrl'] is String ? raw['iconUrl']! as String : null,
        compatibility: raw['compatibility'] is String ? raw['compatibility']! as String : null,
        index: raw['index'] is num ? (raw['index']! as num).toInt() : 0,
      );
}

class BridgeResult {
  const BridgeResult(
    this.ok,
    this.message, {
    this.changed = false,
    this.pending = false,
    this.needsActivity = false,
  });
  final bool ok;
  final String message;
  final bool changed;
  /// A real long-running operation has started; the UI shows a cancellable
  /// loading state and polls the snapshot until the native worker finishes.
  final bool pending;
  /// The native host handed off to the real game Activity.
  final bool needsActivity;

  factory BridgeResult.fromJson(Object? raw) {
    if (raw is Map<Object?, Object?>) {
      return BridgeResult(
        raw['ok'] == true,
        raw['message'] is String ? raw['message']! as String : '',
        changed: raw['changed'] == true,
        pending: raw['pending'] == true,
        needsActivity: raw['needsActivity'] == true,
      );
    }
    return const BridgeResult(false, 'Malformed native reply');
  }
}
