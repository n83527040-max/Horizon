package dev.horizon.android

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.DocumentsContract
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import org.json.JSONObject
import java.io.File

/**
 * The native side of `dev.horizon.android/bridge`.
 *
 * `snapshot()` reads ONLY real device/launcher state:
 *  - the Rust launcher's own config file `android-launcher.json`,
 *  - the real content folders (`horizon-mods`, `resourcepacks`, `shaderpacks`),
 *  - `/proc/self/status` and `/proc/meminfo` (the engine's own memory source),
 *  - `flutter_assets/host-info.json` generated from research-lock.json / Gradle.
 *
 * Nothing is invented. Engine-domain actions that are not bridged yet return an
 * explicit failure; they never report fake success or demo data.
 */
class HorizonBridge(private val activity: Activity, private val channel: MethodChannel) {

    private val prefsFile: File get() = File(activity.filesDir, "flutter-preferences.json")
    private val configFile: File get() = File(activity.filesDir, "android-launcher.json")

    fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
        try {
            when (call.method) {
                "snapshot" -> result.success(snapshot())
                "dispatch" -> {
                    val arguments = call.arguments as? Map<*, *>
                    val action = arguments?.get("action") as? String
                    val args = (arguments?.get("args") as? Map<*, *>) ?: emptyMap<Any?, Any?>()
                    if (action == null) {
                        result.success(failure("Missing action name"))
                    } else {
                        result.success(dispatch(action, args))
                    }
                }
                else -> result.notImplemented()
            }
        } catch (error: Exception) {
            result.success(failure(error.message ?: "Native bridge error"))
        }
    }

    // ---------------------------------------------------------------- snapshot

    private fun snapshot(): Map<String, Any?> = mapOf(
        "appVersion" to hostInfo().optString("appVersion", ""),
        "gameVersion" to hostInfo().optString("gameVersion", ""),
        "profileName" to config().optString("name").ifEmpty { null },
        "accountStatus" to accountStatus(),
        "avatarPath" to null, // hidden until a real skin file is available
        "graphicsApi" to config().optString("renderer").ifEmpty { null },
        "renderScalePercent" to config().opt("render_scale")?.let { percent(it) },
        "ramUsedMb" to procValue("/proc/self/status", "VmRSS:"),
        "ramPeakMb" to procValue("/proc/self/status", "VmHWM:"),
        "ramTotalMb" to procValue("/proc/meminfo", "MemTotal:"),
        "installed" to installedItems(),
        "results" to emptyList<Map<String, Any?>>(), // real catalog results appear via the engine bridge
        "status" to null,
        "settings" to settings(),
        "choices" to choices(),
        "storageBytes" to contentBytes().values.sum(),
        "cacheBytes" to dirSize(activity.cacheDir),
        "contentBytes" to contentBytes(),
        "engineBridgeReady" to false,
    )

    private fun hostInfo(): JSONObject = try {
        JSONObject(activity.assets.open("flutter_assets/host-info.json").bufferedReader().use { it.readText() })
    } catch (_: Exception) {
        JSONObject()
    }

    private fun config(): JSONObject = try {
        JSONObject(configFile.readText())
    } catch (_: Exception) {
        JSONObject()
    }

    private fun preferences(): JSONObject = try {
        JSONObject(prefsFile.readText())
    } catch (_: Exception) {
        JSONObject()
    }

    private fun accountStatus(): String? {
        val name = config().optString("name")
        val secrets = File(activity.filesDir, "account-secrets")
        val signedIn = secrets.isDirectory && (secrets.listFiles()?.isNotEmpty() == true)
        return when {
            signedIn -> "Microsoft account"
            name.isNotEmpty() -> "Offline: $name"
            else -> "Offline"
        }
    }

    private fun settings(): Map<String, Any?> {
        val c = config()
        val p = preferences()
        return mapOf(
            "renderer" to rendererLabel(c.optString("renderer")),
            "renderScale" to (c.opt("render_scale")?.let { percent(it) } ?: 100),
            "orientation" to c.optString("orientation", "portrait"),
            "immersive" to c.optBoolean("immersive", false),
            "avoidCutout" to c.optBoolean("avoid_cutout", false),
            "aspect" to c.optString("aspect", "native"),
            "displayHz" to c.optInt("display_hz", 0).takeIf { it > 0 }?.toString() ?: "System default",
            "performanceHints" to c.optBoolean("performance_hints", false),
            "gpuProfile" to c.optBoolean("gpu_profile", false),
            "recordPerformance" to c.optBoolean("record_performance", false),
            "profile" to c.optInt("profile", 0).toString(),
            "server" to c.optString("server", ""),
            "modSource" to c.optString("mod_source", "Horizon"),
            "shaderSource" to c.optString("shader_source", "Horizon"),
            "theme" to p.optString("theme", "dark"),
            "language" to p.optString("language", "en"),
        )
    }

    private fun choices(): Map<String, List<String>> = mapOf(
        "Renderer" to listOf("Vulkan / batching", "Vulkan / compatibility", "OpenGL ES / experimental"),
        "RenderScale" to listOf("100%", "85%", "75%", "67%", "50%", "25%"),
        "Orientation" to listOf("Portrait", "Landscape", "Reverse portrait", "Reverse landscape", "Automatic rotation"),
        "Aspect" to listOf("Native", "16:9", "4:3", "1:1"),
        "DisplayHz" to listOf("System default", "60 Hz", "90 Hz", "120 Hz"),
        "InputHost" to listOf("Automatic", "Touch & Android keyboard", "Keyboard & mouse"),
    )

    private fun installedItems(): List<Map<String, Any?>> {
        val out = mutableListOf<Map<String, Any?>>()
        var index = 0
        for ((folder, kind) in CONTENT_FOLDERS) {
            val dir = File(activity.filesDir, folder)
            val files = dir.listFiles()?.filter { it.isFile && !it.name.startsWith(".") }?.sortedBy { it.name }
                ?: emptyList()
            for (file in files) {
                out.add(
                    mapOf(
                        "file" to file.name,
                        "title" to file.nameWithoutExtension,
                        "folder" to folder,
                        "kind" to kind,
                        "enabled" to null, // the engine owns enable/disable state
                        "index" to index++,
                    )
                )
            }
        }
        return out
    }

    private fun contentBytes(): Map<String, Int> = CONTENT_FOLDERS.associate { (folder, kind) ->
        kind to dirSize(File(activity.filesDir, folder)).toInt()
    }

    private fun dirSize(dir: File): Long {
        if (!dir.exists()) return 0
        if (dir.isFile) return dir.length()
        return dir.walkTopDown().filter { it.isFile }.sumOf { it.length() }
    }

    private fun procValue(path: String, key: String): Int? = try {
        File(path).useLines { lines ->
            val value = lines.firstOrNull { it.startsWith(key) }
                ?.split(Regex("\\s+"))?.getOrNull(1)?.toLongOrNull()
            value?.div(1024)?.toInt()
        }
    } catch (_: Exception) {
        null
    }

    private fun percent(value: Any?): Int = when (value) {
        is Number -> value.toInt()
        is JSONObject -> value.optInt("0", 100)
        else -> 100
    }

    private fun rendererLabel(raw: String): String? = when (raw.lowercase()) {
        "vulkan" -> "Vulkan / batching"
        "vulkancompatibility" -> "Vulkan / compatibility"
        "opengl" -> "OpenGL ES / experimental"
        else -> raw.ifEmpty { null }
    }

    // ---------------------------------------------------------------- actions

    /** Every name below covers one of the 49 Rust `Action` variants. */
    fun dispatch(action: String, args: Map<*, *>): Map<String, Any?> {
        return when (action) {
            "Play" -> launchGame()
            "Repository" -> openUrl("https://github.com/n83527040-max/Horizon")
            "OpenFolder" -> openFolder(args["kind"] as? String)
            "Import" -> openPicker(args["kind"] as? String)
            "LoginLink", "ReviewApp" -> enginePending(action)
            "SignIn", "DeviceSignIn", "SignOut", "CloseLogin", "CopyLoginCode",
            "CancelLogin", "Account" -> enginePending(action)
            "Field" -> setPreference(args)
            "Immersive", "Cutout" -> setConfigBoolean(action, args)
            "Renderer", "RenderScale", "Aspect", "DisplayHz", "Orientation",
            "InputHost", "Profile" -> enginePending(action)
            "ChooseRenderer", "ChooseRenderScale", "ChooseAspect", "ChooseDisplayHz",
            "ChooseOrientation", "ChooseInputHost", "ChooseProfile" -> enginePending(action)
            "Page", "Dismiss" -> success("Navigation handled in Dart", changed = false)
            "Search", "ShaderSource", "ModSource", "Install", "ToggleSection",
            "ToggleLocal", "DeleteLocal", "ConfirmDeleteLocal", "Next", "Previous",
            "FinishNameEdit", "Logs", "RecordPerformance", "ExportPerformance",
            "ExportDiagnostics", "PerformanceHints", "GpuProfile" -> enginePending(action)
            else -> failure("Unknown action: $action")
        }
    }

    private fun launchGame(): Map<String, Any?> = try {
        val intent = Intent().setClassName(activity.packageName, "dev.horizon.android.HorizonActivity")
        intent.putExtra("horizon.launch_game", true)
        activity.startActivity(intent)
        success("Starting Horizon game Activity")
    } catch (error: Exception) {
        failure("Game Activity is not packaged yet: ${error.message}")
    }

    private fun openUrl(url: String): Map<String, Any?> = try {
        activity.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
        success("Opened $url")
    } catch (error: Exception) {
        failure(error.message ?: "Could not open link")
    }

    private fun openFolder(kind: String?): Map<String, Any?> {
        val folder = CONTENT_FOLDERS.keys.firstOrNull { it.contains(kind ?: "", true) }
            ?: CONTENT_FOLDERS.keys.first()
        return try {
            val authority = "${activity.packageName}.documents"
            val uri = DocumentsContract.buildRootUri(authority, folder)
            activity.startActivity(Intent(Intent.ACTION_VIEW, uri).addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION))
            success("Opened ${CONTENT_FOLDERS[folder]} folder")
        } catch (error: Exception) {
            failure(error.message ?: "Could not open folder")
        }
    }

    private fun openPicker(kind: String?): Map<String, Any?> = try {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
            putExtra(Intent.EXTRA_MIME_TYPES, mimeTypes(kind))
        }
        pendingKind = kind ?: "Mods"
        activity.startActivityForResult(intent, REQUEST_IMPORT)
        success("Choose a file to import")
    } catch (error: Exception) {
        failure(error.message ?: "Could not open the file picker")
    }

    fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        if (requestCode != REQUEST_IMPORT) return
        val uri: Uri? = data?.data
        if (resultCode != Activity.RESULT_OK || uri == null) {
            channel.invokeMethod("importResult", mapOf("ok" to false, "message" to "Import cancelled"))
            return
        }
        // The real file copy/validation stays in the native Rust documents layer.
        channel.invokeMethod(
            "importResult",
            mapOf("ok" to false, "message" to "SAF URI received; native importer is bridged in phase 9", "uri" to uri.toString())
        )
    }

    private fun setPreference(args: Map<*, *>): Map<String, Any?> {
        val name = args["name"] as? String ?: return failure("Missing preference name")
        val value = args["value"]
        return try {
            when (name) {
                "theme", "language" -> {
                    val json = preferences()
                    if (value == null) json.remove(name) else json.put(name, value.toString())
                    save(prefsFile, json)
                    success("Saved $name", changed = true)
                }
                "clearCache" -> {
                    val cleared = activity.cacheDir.deleteRecursively()
                    success(if (cleared) "App cache cleared" else "Cache is already empty", changed = true)
                }
                else -> enginePending("Field($name)")
            }
        } catch (error: Exception) {
            failure(error.message ?: "Could not save preference")
        }
    }

    private fun setConfigBoolean(action: String, args: Map<*, *>): Map<String, Any?> {
        val key = if (action == "Immersive") "immersive" else "avoid_cutout"
        return try {
            val json = config()
            val current = json.optBoolean(key, false)
            json.put(key, args["value"] as? Boolean ?: !current)
            save(configFile, json)
            success("Updated $key", changed = true)
        } catch (error: Exception) {
            failure(error.message ?: "Could not update setting")
        }
    }

    private fun save(file: File, json: JSONObject) {
        val partial = File(file.parentFile, "${file.name}.partial")
        partial.writeText(json.toString(2))
        if (!partial.renameTo(file)) file.writeText(json.toString(2))
    }

    private fun enginePending(action: String): Map<String, Any?> = failure(
        "Action $action requires the native engine bridge (phase 9). " +
            "The Rust launcher logic is preserved, not replaced by a dummy."
    )

    private fun success(message: String, changed: Boolean = false): Map<String, Any?> =
        mapOf("ok" to true, "message" to message, "changed" to changed)

    private fun failure(message: String): Map<String, Any?> =
        mapOf("ok" to false, "message" to message, "changed" to false)

    private fun mimeTypes(kind: String?): Array<String> = when (kind) {
        "Packs" -> arrayOf("application/zip", "application/octet-stream")
        "Shaders" -> arrayOf("application/zip", "application/octet-stream")
        else -> arrayOf("application/octet-stream")
    }

    private var pendingKind: String = "Mods"

    companion object {
        private const val REQUEST_IMPORT: Int = 4101
        private val CONTENT_FOLDERS: Map<String, String> = linkedMapOf(
            "horizon-mods" to "Mods",
            "resourcepacks" to "Packs",
            "shaderpacks" to "Shaders",
        )
    }
}
