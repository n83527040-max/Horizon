package dev.horizon.android

import android.app.Activity
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Handler
import android.os.Looper
import android.provider.DocumentsContract
import android.provider.OpenableColumns
import android.util.Log
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

/**
 * `dev.horizon.android/bridge`: Flutter UI <-> JNI/C ABI <-> real Rust engine.
 *
 * All stateful work is delegated to HorizonEngine (Rust). This class only
 * supplies what Rust cannot have outside a NativeActivity: an Android Context,
 * SAF document picking, link opening, the clipboard and the game Activity
 * hand-off. Every result is a real outcome; no placeholder remains.
 */
class HorizonBridge(private val activity: Activity, private val channel: MethodChannel) {

    private val executor = Executors.newSingleThreadExecutor { Thread(it, "horizon-bridge") }
    private val main = Handler(Looper.getMainLooper())
    private val stopped = AtomicBoolean(false)
    private var pendingKind: String = "Mods"

    init {
        val reply = HorizonEngine.nativeInit(
            activity.filesDir.absolutePath,
            activity.cacheDir.absolutePath,
        )
        Log.i(TAG, "HorizonEngine nativeInit: $reply")
    }

    fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
        when (call.method) {
            "snapshot" -> async(result) { parse(HorizonEngine.nativeSnapshot()) }
            "cancel" -> async(result) { parse(HorizonEngine.nativeCancel()) }
            "dispatch" -> {
                val arguments = call.arguments as? Map<*, *>
                val action = arguments?.get("action") as? String
                val args = (arguments?.get("args") as? Map<*, *>) ?: emptyMap<Any?, Any?>()
                if (action == null) {
                    result.success(failure("Missing action name"))
                } else {
                    dispatch(action, args, result)
                }
            }
            else -> result.notImplemented()
        }
    }

    private fun dispatch(action: String, args: Map<*, *>, result: MethodChannel.Result) {
        when (action) {
            "Play" -> {
                startGame()
                result.success(success("Starting Minecraft in the native engine", pending = true))
            }
            "Repository" -> result.success(openUrl(REPOSITORY))
            "ReviewApp" -> result.success(openUrl(REVIEW_URL))
            "OpenFolder" -> result.success(openFolder(args["kind"] as? String))
            "Import" -> result.success(openPicker(args["kind"] as? String))
            else -> async(result) {
                val reply = parse(HorizonEngine.nativeDispatch(action, toJson(args).toString()))
                applyReply(reply)
                reply
            }
        }
    }

    /** Rust may ask the Android host to open a real link or copy a real code. */
    private fun applyReply(reply: Map<String, Any?>) {
        (reply["url"] as? String)?.let { url -> main.post { openUrl(url) } }
        (reply["code"] as? String)?.let { code -> main.post { copyCode(code) } }
    }

    private fun async(result: MethodChannel.Result, block: () -> Map<String, Any?>) {
        executor.execute {
            val value = try {
                block()
            } catch (error: Throwable) {
                failure(error.message ?: "Native bridge error")
            }
            if (!stopped.get()) main.post { result.success(value) }
        }
    }

    // ------------------------------------------------------------- engine hand-off

    private fun startGame() {
        try {
            val intent = Intent().setClassName(activity.packageName, "dev.horizon.android.HorizonActivity")
            intent.putExtra("horizon.launch_game", true)
            intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            activity.startActivity(intent)
        } catch (error: Exception) {
            emit("gameLaunch", failure("Cannot start the native game Activity: ${error.message}"))
        }
    }

    private fun openUrl(url: String): Map<String, Any?> = try {
        activity.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
        success("Opened $url")
    } catch (error: Exception) {
        failure(error.message ?: "Could not open the link")
    }

    private fun copyCode(code: String) {
        try {
            val manager = activity.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
            manager.setPrimaryClip(ClipData.newPlainText("Horizon sign-in code", code))
            emit("loginCode", success("Sign-in code copied: $code"))
        } catch (error: Exception) {
            emit("loginCode", failure(error.message ?: "Could not copy the code"))
        }
    }

    private fun openFolder(kind: String?): Map<String, Any?> {
        val folder = folderName(kind)
        return try {
            val uri = DocumentsContract.buildRootUri("${activity.packageName}.documents", folder)
            activity.startActivity(
                Intent(Intent.ACTION_VIEW, uri).addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            )
            success("Opened the real $folder folder")
        } catch (error: Exception) {
            failure(error.message ?: "Could not open the content folder")
        }
    }

    // ------------------------------------------------------------- real SAF import

    private fun openPicker(kind: String?): Map<String, Any?> = try {
        pendingKind = normalizeKind(kind)
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
            putExtra(Intent.EXTRA_MIME_TYPES, arrayOf("application/zip", "application/octet-stream", "*/*"))
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        }
        activity.startActivityForResult(intent, REQUEST_IMPORT)
        success("Choose a .zip / .hshader / .hmod file", pending = true)
    } catch (error: Exception) {
        failure(error.message ?: "Could not open the document picker")
    }

    fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        if (requestCode != REQUEST_IMPORT) return
        val uri: Uri? = data?.data
        if (resultCode != Activity.RESULT_OK || uri == null) {
            emit("import", failure("Import cancelled"))
            return
        }
        val kind = pendingKind
        executor.execute {
            val reply = try {
                importDocument(kind, uri)
            } catch (error: Throwable) {
                failure(error.message ?: "Import failed")
            }
            main.post { emit("import", reply) }
        }
    }

    /** Copies the granted document through ContentResolver; no path is inferred
     *  from its URI. Rust then reuses the engine's own validation and rename. */
    private fun importDocument(kind: String, uri: Uri): Map<String, Any?> {
        val resolver = activity.contentResolver
        var name = "import.bin"
        resolver.query(uri, null, null, null, null)?.use { cursor ->
            val column = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            if (column >= 0 && cursor.moveToFirst()) {
                name = cursor.getString(column) ?: name
            }
        }
        val staged = File(activity.cacheDir, "content-imports/.import-${System.nanoTime()}.part")
        staged.parentFile?.mkdirs()
        val input = resolver.openInputStream(uri)
            ?: throw IllegalArgumentException("Cannot read the selected document")
        input.use { source ->
            staged.outputStream().use { output ->
                val buffer = ByteArray(65536)
                var total = 0L
                while (true) {
                    val read = source.read(buffer)
                    if (read < 0) break
                    total += read
                    if (total > MAX_IMPORT_BYTES) throw IllegalArgumentException("Import exceeds 256 MiB")
                    output.write(buffer, 0, read)
                }
            }
        }
        return parse(HorizonEngine.nativeImportStaged(kind, staged.absolutePath, name))
    }

    private fun emit(type: String, reply: Map<String, Any?>) {
        if (stopped.get()) return
        val event = HashMap<String, Any?>(reply)
        event["type"] = type
        channel.invokeMethod("nativeEvent", event)
    }

    /** Stops the Rust poller and the executor before the Activity is destroyed. */
    fun shutdown() {
        if (!stopped.compareAndSet(false, true)) return
        try {
            HorizonEngine.nativeShutdown()
        } catch (error: Throwable) {
            Log.w(TAG, "nativeShutdown: ${error.message}")
        }
        executor.shutdownNow()
    }

    // ------------------------------------------------------------- JSON plumbing

    private fun parse(raw: String): Map<String, Any?> = try {
        when (val value = fromJson(JSONObject(raw))) {
            is Map<*, *> -> value as Map<String, Any?>
            else -> failure("Malformed native reply")
        }
    } catch (error: Throwable) {
        failure("Malformed native reply: ${error.message}")
    }

    private fun fromJson(value: Any?): Any? = when (value) {
        JSONObject.NULL -> null
        is JSONObject -> {
            val map = HashMap<String, Any?>()
            val keys = value.keys()
            while (keys.hasNext()) {
                val key = keys.next()
                map[key] = fromJson(value.get(key))
            }
            map
        }
        is JSONArray -> (0 until value.length()).map { fromJson(value.get(it)) }
        else -> value
    }

    private fun toJson(map: Map<*, *>): JSONObject {
        val json = JSONObject()
        for ((key, value) in map) {
            if (key is String) json.put(key, jsonValue(value))
        }
        return json
    }

    private fun jsonValue(value: Any?): Any = when (value) {
        null -> JSONObject.NULL
        is Map<*, *> -> toJson(value)
        is List<*> -> JSONArray(value.map { jsonValue(it) })
        else -> value
    }

    private fun success(message: String, changed: Boolean = false, pending: Boolean = false) =
        mapOf("ok" to true, "message" to message, "changed" to changed, "pending" to pending)

    private fun failure(message: String) =
        mapOf("ok" to false, "message" to message, "changed" to false, "pending" to false)

    private fun normalizeKind(kind: String?): String = when (kind) {
        "Packs", "resourcepacks" -> "Packs"
        "Shaders", "shaderpacks" -> "Shaders"
        else -> "Mods"
    }

    private fun folderName(kind: String?): String = when (normalizeKind(kind)) {
        "Packs" -> "resourcepacks"
        "Shaders" -> "shaderpacks"
        else -> "horizon-mods"
    }

    companion object {
        private const val TAG = "HorizonBridge"
        private const val REQUEST_IMPORT = 4101
        private const val MAX_IMPORT_BYTES = 256L * 1024 * 1024
        private const val REPOSITORY = "https://github.com/n83527040-max/Horizon"
        private const val REVIEW_URL = "https://aka.ms/mce-reviewappid"
    }
}
