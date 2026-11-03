package dev.horizon.android

/**
 * The JNI/C ABI door into the real Rust `libhorizon_android.so`.
 *
 * The symbols live in `crates/horizon-client/src/android/flutter_bridge.rs`
 * (the hosting crate root, not the simulation). Nothing here invents data:
 * every reply comes from `Catalog`, `Config`, `Accounts`, `SignIn`,
 * `diagnostics` or `content_files` inside the original engine.
 */
object HorizonEngine {
    init {
        System.loadLibrary("horizon_android")
    }

    /** Installs the JavaVM, the real filesDir/cacheDir and starts the worker poller. */
    external fun nativeInit(home: String, cache: String): String

    /** Real launcher state; never a cached or demo snapshot. */
    external fun nativeSnapshot(): String

    /** Executes one of the 49 real actions; returns a JSON result. */
    external fun nativeDispatch(action: String, args: String): String

    /** Cancels sign-in and detaches pending catalog interest. */
    external fun nativeCancel(): String

    /** Starts the REAL `official_install::install_with_progress` worker. */
    external fun nativeInstallStart(): String

    /** Stops the Rust poller thread before the Activity dies. */
    external fun nativeShutdown(): String

    /** Finishes a real SAF import using the engine's own archive validation. */
    external fun nativeImportStaged(kind: String, staged: String, name: String): String
}
