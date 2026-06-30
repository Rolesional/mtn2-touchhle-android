/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Parts of this file are derived from SDL 2's Android project template, which
 * has a different license. Please see vendor/SDL/LICENSE.txt for details.
 */
package org.touchhle.android;

import android.graphics.Color;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.util.Log;
import android.util.TypedValue;
import android.view.Gravity;
import android.widget.FrameLayout;
import android.widget.ImageView;
import android.widget.TextView;
import org.libsdl.app.SDLActivity;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;

public class MainActivity extends SDLActivity {
    private static final String TAG = "touchHLE";
    private static final String IPA_ASSET = "MonsterTrucksNitroV120.ipa";
    private static final String IPA_FILENAME = "MonsterTrucksNitroV120.ipa";
    private static final String SANDBOX_PREFS_ASSET =
        "sandbox/com.redlynx.MTN2/Library/Preferences/com.redlynx.MTN2.plist";
    private static final long PORT_BANNER_MS = 5000;
    private static final long SPLASH_MS = 5000;

    private File ipaPath;
    private FrameLayout splashOverlay;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        prepopulateUserDataDir();
        try {
            super.onCreate(savedInstanceState);
        } catch (Throwable t) {
            Log.e(TAG, "onCreate failed", t);
            appendLog("MainActivity.onCreate failed: " + t);
            throw t;
        }
        showSplashOverlay();
    }

    @Override
    protected String[] getArguments() {
        if (ipaPath == null || !ipaPath.exists()) {
            prepopulateUserDataDir();
        }
        if (ipaPath == null || !ipaPath.exists()) {
            Log.e(TAG, "Bundled IPA missing at launch");
            appendLog("Bundled IPA missing at launch");
            return new String[0];
        }
        return new String[]{
            ipaPath.getAbsolutePath(),
            "--allow-network-access"
        };
    }

    private File getStorageBase() {
        File base = getExternalFilesDir(null);
        if (base == null) {
            base = getFilesDir();
        }
        return base;
    }

    private void prepopulateUserDataDir() {
        File base = getStorageBase();
        if (base == null) {
            Log.e(TAG, "No writable storage directory");
            return;
        }
        writeFileIfMissing(new File(base, "touchHLE_log.txt"), "touchHLE Android startup\n");
        File apps = new File(base, "touchHLE_apps");
        if (!apps.exists() && !apps.mkdirs()) {
            Log.e(TAG, "Couldn't create touchHLE_apps");
            return;
        }
        ipaPath = new File(apps, IPA_FILENAME);
        extractBundledIpa(ipaPath);
        extractSandboxPrefs(base);
        writeOptionsFile(new File(base, "touchHLE_options.txt"));
    }

    private void extractSandboxPrefs(File base) {
        File dest = new File(
            base,
            "touchHLE_sandbox/com.redlynx.MTN2/Library/Preferences/com.redlynx.MTN2.plist"
        );
        extractAssetIfMissing(SANDBOX_PREFS_ASSET, dest);
    }

    private void extractAssetIfMissing(String assetPath, File dest) {
        if (dest.exists() && dest.length() > 0) {
            return;
        }
        File parent = dest.getParentFile();
        if (parent != null && !parent.exists() && !parent.mkdirs()) {
            Log.e(TAG, "Couldn't create " + parent.getAbsolutePath());
            return;
        }
        try (InputStream in = getAssets().open(assetPath);
             FileOutputStream out = new FileOutputStream(dest)) {
            byte[] buffer = new byte[8192];
            int read;
            while ((read = in.read(buffer)) != -1) {
                out.write(buffer, 0, read);
            }
            Log.i(TAG, "Extracted sandbox asset to " + dest.getAbsolutePath());
        } catch (IOException e) {
            Log.e(TAG, "Failed to extract sandbox asset " + assetPath, e);
            appendLog("Failed to extract sandbox asset " + assetPath + ": " + e);
        }
    }

    private void writeOptionsFile(File file) {
        String content = "com.redlynx.MTN2: --allow-network-access\n";
        File parent = file.getParentFile();
        if (parent != null && !parent.exists() && !parent.mkdirs()) {
            Log.e(TAG, "Couldn't create " + parent.getAbsolutePath());
            return;
        }
        try (FileOutputStream out = new FileOutputStream(file)) {
            out.write(content.getBytes(StandardCharsets.UTF_8));
        } catch (IOException e) {
            Log.e(TAG, "Couldn't write " + file.getAbsolutePath(), e);
        }
    }

    private void extractBundledIpa(File dest) {
        if (dest.exists() && dest.length() > 0) {
            return;
        }
        File parent = dest.getParentFile();
        if (parent != null && !parent.exists() && !parent.mkdirs()) {
            Log.e(TAG, "Couldn't create " + parent.getAbsolutePath());
            return;
        }
        try (InputStream in = getAssets().open(IPA_ASSET);
             FileOutputStream out = new FileOutputStream(dest)) {
            byte[] buffer = new byte[65536];
            int read;
            while ((read = in.read(buffer)) != -1) {
                out.write(buffer, 0, read);
            }
            Log.i(TAG, "Extracted bundled IPA to " + dest.getAbsolutePath());
        } catch (IOException e) {
            Log.e(TAG, "Failed to extract bundled IPA", e);
            appendLog("Failed to extract bundled IPA: " + e);
        }
    }

    /** Fullscreen splash while touchHLE loads underneath. */
    private void showSplashOverlay() {
        splashOverlay = new FrameLayout(this);
        splashOverlay.setBackgroundColor(Color.BLACK);

        ImageView image = new ImageView(this);
        image.setScaleType(ImageView.ScaleType.FIT_XY);
        image.setImageResource(R.drawable.splash_mtn2);
        splashOverlay.addView(image, new FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.MATCH_PARENT,
            FrameLayout.LayoutParams.MATCH_PARENT
        ));

        addContentView(splashOverlay, new FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.MATCH_PARENT,
            FrameLayout.LayoutParams.MATCH_PARENT
        ));

        new Handler(Looper.getMainLooper()).postDelayed(this::dismissSplashOverlay, SPLASH_MS);
    }

    private void dismissSplashOverlay() {
        if (splashOverlay != null) {
            android.view.ViewParent parent = splashOverlay.getParent();
            if (parent instanceof android.view.ViewGroup) {
                ((android.view.ViewGroup) parent).removeView(splashOverlay);
            }
            splashOverlay = null;
        }
        showPortCreditBanner();
    }

    private void showPortCreditBanner() {
        TextView banner = new TextView(this);
        banner.setText("Ported by Rolesional");
        banner.setTextColor(Color.WHITE);
        banner.setTextSize(TypedValue.COMPLEX_UNIT_SP, 14);
        banner.setBackgroundColor(0xE6333333);
        int padH = (int) TypedValue.applyDimension(
            TypedValue.COMPLEX_UNIT_DIP, 20, getResources().getDisplayMetrics());
        int padV = (int) TypedValue.applyDimension(
            TypedValue.COMPLEX_UNIT_DIP, 12, getResources().getDisplayMetrics());
        banner.setPadding(padH, padV, padH, padV);
        banner.setGravity(Gravity.CENTER);

        FrameLayout.LayoutParams params = new FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.WRAP_CONTENT,
            FrameLayout.LayoutParams.WRAP_CONTENT,
            Gravity.BOTTOM | Gravity.CENTER_HORIZONTAL
        );
        int margin = (int) TypedValue.applyDimension(
            TypedValue.COMPLEX_UNIT_DIP, 24, getResources().getDisplayMetrics());
        params.setMargins(margin, 0, margin, margin);
        addContentView(banner, params);

        new Handler(Looper.getMainLooper()).postDelayed(() -> {
            android.view.ViewParent parent = banner.getParent();
            if (parent instanceof android.view.ViewGroup) {
                ((android.view.ViewGroup) parent).removeView(banner);
            }
        }, PORT_BANNER_MS);
    }

    private static void writeFileIfMissing(File file, String content) {
        if (file.exists()) {
            return;
        }
        File parent = file.getParentFile();
        if (parent != null && !parent.exists() && !parent.mkdirs()) {
            Log.e(TAG, "Couldn't create " + parent.getAbsolutePath());
            return;
        }
        try (FileOutputStream out = new FileOutputStream(file)) {
            out.write(content.getBytes(StandardCharsets.UTF_8));
        } catch (IOException e) {
            Log.e(TAG, "Couldn't write " + file.getAbsolutePath(), e);
        }
    }

    private void appendLog(String line) {
        File base = getStorageBase();
        if (base == null) {
            return;
        }
        File log = new File(base, "touchHLE_log.txt");
        try (FileOutputStream out = new FileOutputStream(log, true)) {
            out.write((line + "\n").getBytes(StandardCharsets.UTF_8));
        } catch (IOException e) {
            Log.e(TAG, "Couldn't append to log", e);
        }
    }

    @Override
    protected String[] getLibraries() {
        return new String[]{
            "SDL2",
            "touchHLE"
        };
    }
}
