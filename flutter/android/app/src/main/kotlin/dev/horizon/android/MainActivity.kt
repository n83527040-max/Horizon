package dev.horizon.android

import android.content.Intent
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

/**
 * The real Android entry point for the Flutter launcher.
 *
 * The game itself still runs in the stock NativeActivity `HorizonActivity`
 * (libhorizon_android.so / wgpu / Vulkan). This Activity only hosts Dart and
 * forwards verified actions to the native layer.
 */
class MainActivity : FlutterActivity() {

    private var bridge: HorizonBridge? = null

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        val channel = MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL)
        bridge = HorizonBridge(this, channel)
        channel.setMethodCallHandler { call, result -> bridge!!.onMethodCall(call, result) }
    }

    @Deprecated("Deprecated in Java")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        bridge?.onActivityResult(requestCode, resultCode, data)
    }

    override fun onDestroy() {
        // Rust owns one poller thread and the sign-in task. Stop both before the
        // Activity dies so neither leaks a JNI reference or a detached thread.
        bridge?.shutdown()
        bridge = null
        super.onDestroy()
    }

    companion object {
        const val CHANNEL: String = "dev.horizon.android/bridge"
    }
}
