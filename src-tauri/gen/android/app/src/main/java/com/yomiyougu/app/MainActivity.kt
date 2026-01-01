package com.yomiyougu.app

import android.os.Bundle
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat

class MainActivity : TauriActivity() {
  private lateinit var insetsController: WindowInsetsControllerCompat
  
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    
    WindowCompat.setDecorFitsSystemWindows(window, false)
    insetsController = WindowInsetsControllerCompat(window, window.decorView)
    insetsController.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
  }
  
  override fun onWebViewCreate(webView: WebView) {
    webView.addJavascriptInterface(FullscreenInterface(), "AndroidFullscreen")
  }
  
  inner class FullscreenInterface {
    @JavascriptInterface
    fun setFullscreen(enabled: Boolean) {
      runOnUiThread {
        if (enabled) {
          insetsController.hide(WindowInsetsCompat.Type.systemBars())
        } else {
          insetsController.show(WindowInsetsCompat.Type.systemBars())
        }
      }
    }
  }
}
