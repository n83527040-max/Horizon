/// Typed contract for every real launcher action in the Rust host.
///
/// Each name matches a variant of `enum Action` in
/// `crates/horizon-client/src/android/launcher_pages.rs`. The Dart UI never
/// invents an action: `test/actions_parity_test.dart` parses the Rust enum and
/// fails the build if this list drifts.
enum HorizonAction {
  Page,
  Play,
  Search,
  ShaderSource,
  ModSource,
  Field,
  FinishNameEdit,
  Install,
  Import,
  OpenFolder,
  ToggleSection,
  ToggleLocal,
  DeleteLocal,
  ConfirmDeleteLocal,
  Next,
  Previous,
  Profile,
  Logs,
  RecordPerformance,
  ExportPerformance,
  ExportDiagnostics,
  Account,
  SignIn,
  DeviceSignIn,
  CopyLoginCode,
  CancelLogin,
  ReviewApp,
  Repository,
  SignOut,
  LoginLink,
  CloseLogin,
  Renderer,
  RenderScale,
  ChooseRenderScale,
  Orientation,
  InputHost,
  ChooseInputHost,
  ChooseOrientation,
  ChooseRenderer,
  ChooseProfile,
  Dismiss,
  Immersive,
  Cutout,
  Aspect,
  ChooseAspect,
  DisplayHz,
  ChooseDisplayHz,
  PerformanceHints,
  GpuProfile,
}

/// The six real pages of the launcher (`enum Page` in catalog.rs).
enum HorizonPage { Play, Mods, Packs, Shaders, Installed, Settings }

/// The three real content kinds (`enum Kind` in content_files.rs).
enum ContentKind {
  Mods('horizon-mods'),
  Packs('resourcepacks'),
  Shaders('shaderpacks');

  const ContentKind(this.folder);
  final String folder;

  static ContentKind? fromFolder(String? value) {
    for (final ContentKind kind in values) {
      if (kind.folder == value) return kind;
    }
    return null;
  }
}

/// The two real catalog sources (`enum SourceId` in launcher/src/browser.rs).
enum CatalogSource { Horizon, Modrinth }

/// Native setting keys. These come from `Config` / `Settings` in catalog.rs and
/// are surfaced verbatim; no setting is invented by the UI.
abstract final class SettingKeys {
  static const String renderer = 'renderer';
  static const String renderScale = 'renderScale';
  static const String orientation = 'orientation';
  static const String immersive = 'immersive';
  static const String avoidCutout = 'avoidCutout';
  static const String aspect = 'aspect';
  static const String displayHz = 'displayHz';
  static const String performanceHints = 'performanceHints';
  static const String gpuProfile = 'gpuProfile';
  static const String recordPerformance = 'recordPerformance';
  static const String profile = 'profile';
  static const String inputHost = 'inputHost';
  static const String server = 'server';
  static const String modSource = 'modSource';
  static const String shaderSource = 'shaderSource';
  // New, genuinely persisted Flutter-side preferences (stored separately from
  // the Rust config file so the two writers never race).
  static const String theme = 'theme';
  static const String language = 'language';
}
