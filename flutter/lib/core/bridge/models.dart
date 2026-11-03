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
    this.catalogStatus,
    this.errorKind = 'none',
    this.canRetry = false,
    this.authStage,
    this.progress,
    this.activePage,
    this.filterVersion,
    this.filterLoader,
    this.accounts = const <LauncherAccount>[],
    this.installReady = false,
    this.installActive = false,
    this.installStage,
    this.installDone = 0,
    this.installTotal = 0,
    this.installPercent,
    this.installSpeedBps,
    this.installEtaSeconds,
    this.installMilestone = 0,
    this.installError,
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
  /// REAL native status text; never replaced by a generic "No results".
  final String? catalogStatus;
  /// network | api | parse | compatibility | storage | auth | other | none
  final String errorKind;
  final bool canRetry;
  final String? authStage;
  /// Real download progress 0..1, null when no download is running.
  final double? progress;
  final String? activePage;
  final String? filterVersion;
  final String? filterLoader;
  final List<LauncherAccount> accounts;

  /// REAL Minecraft resource installer state. Nullable values are genuinely
  /// unknown (total size, speed, ETA), never fabricated.
  final bool installReady;
  final bool installActive;
  final String? installStage;
  final int installDone;
  final int installTotal;
  final int? installPercent;
  final int? installSpeedBps;
  final int? installEtaSeconds;
  final int installMilestone;
  final String? installError;

  /// null when the downloader has not supplied a total size yet.
  double? get installFraction =>
      installPercent == null ? null : installPercent! / 100.0;

  bool get hasError => errorKind != 'none' && errorKind != 'other';

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
      catalogStatus: text(raw['catalogStatus']),
      errorKind: text(raw['errorKind']) ?? 'none',
      canRetry: raw['canRetry'] == true,
      authStage: text(raw['authStage']),
      progress: raw['progress'] is num ? (raw['progress']! as num).toDouble() : null,
      activePage: text(raw['activePage']),
      filterVersion: text(raw['filterVersion']),
      filterLoader: text(raw['filterLoader']),
      installReady: raw['installReady'] == true,
      installActive: raw['installActive'] == true,
      installStage: text(raw['installStage']),
      installDone: integer(raw['installDone']) ?? 0,
      installTotal: integer(raw['installTotal']) ?? 0,
      installPercent: integer(raw['installPercent']),
      installSpeedBps: integer(raw['installSpeedBps']),
      installEtaSeconds: integer(raw['installEtaSeconds']),
      installMilestone: integer(raw['installMilestone']) ?? 0,
      installError: text(raw['installError']),
      accounts: list(raw['accounts'])
          .whereType<Map<Object?, Object?>>()
          .map(LauncherAccount.fromJson)
          .toList(growable: false),
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
    this.author,
    this.downloads,
    this.versions = const <String>[],
    this.loaders = const <String>[],
    this.installable = true,
    this.projectId,
    this.pageUrl,
    required this.index,
  });

  final String title;
  final String? description;
  final String? version;
  final String? source;
  final String? iconUrl;
  final String? compatibility;
  final String? author;
  final int? downloads;
  final List<String> versions;
  final List<String> loaders;
  final bool installable;
  final String? projectId;
  final String? pageUrl;
  final int index;

  factory CatalogProject.fromJson(Map<Object?, Object?> raw) {
    List<Object?> list(Object? value) => value is List<Object?> ? value : const <Object?>[];
    return CatalogProject(
        title: raw['title'] is String ? raw['title']! as String : '',
        description: raw['description'] is String ? raw['description']! as String : null,
        version: raw['version'] is String ? raw['version']! as String : null,
        source: raw['source'] is String ? raw['source']! as String : null,
        iconUrl: raw['iconUrl'] is String ? raw['iconUrl']! as String : null,
        compatibility: raw['compatibility'] is String ? raw['compatibility']! as String : null,
        author: raw['author'] is String ? raw['author']! as String : null,
        downloads: raw['downloads'] is num ? (raw['downloads']! as num).toInt() : null,
        versions: list(raw['versions']).whereType<String>().toList(growable: false),
        loaders: list(raw['loaders']).whereType<String>().toList(growable: false),
        installable: raw['installable'] != false,
        projectId: raw['projectId'] is String ? raw['projectId']! as String : null,
        pageUrl: raw['pageUrl'] is String ? raw['pageUrl']! as String : null,
        index: raw['index'] is num ? (raw['index']! as num).toInt() : 0,
    );
  }
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


/// A REAL account from accounts.json (Microsoft or offline). No invented data.
class LauncherAccount {
  const LauncherAccount({
    required this.id,
    required this.name,
    required this.type,
    this.selected = false,
    this.skinUrl,
  });

  final String id;
  final String name;
  final String type; // 'microsoft' | 'offline'
  final bool selected;
  final String? skinUrl;

  bool get isOffline => type == 'offline';

  factory LauncherAccount.fromJson(Map<Object?, Object?> raw) => LauncherAccount(
        id: raw['id'] is String ? raw['id']! as String : '',
        name: raw['name'] is String ? raw['name']! as String : '',
        type: raw['type'] is String ? raw['type']! as String : 'offline',
        selected: raw['selected'] == true,
        skinUrl: raw['skin'] is String ? raw['skin']! as String : null,
      );
}
