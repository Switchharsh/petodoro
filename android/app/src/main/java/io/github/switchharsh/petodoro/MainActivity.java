package io.github.switchharsh.petodoro;

import android.app.Activity;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.graphics.Color;
import android.graphics.Typeface;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.provider.Settings;
import android.view.Gravity;
import android.widget.Button;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;

/** One small screen: grant the overlay permission, then show or hide the pet. */
public class MainActivity extends Activity {
    private Button permissionButton;
    private Button petButton;
    private TextView status;

    @Override
    protected void onCreate(Bundle state) {
        super.onCreate(state);
        int pad = dp(24);

        LinearLayout col = new LinearLayout(this);
        col.setOrientation(LinearLayout.VERTICAL);
        col.setGravity(Gravity.CENTER_HORIZONTAL);
        col.setPadding(pad, pad * 2, pad, pad);
        col.setBackgroundColor(Color.rgb(0xff, 0xfa, 0xf6));

        ImageView logo = new ImageView(this);
        logo.setImageResource(R.mipmap.ic_launcher);
        col.addView(logo, new LinearLayout.LayoutParams(dp(128), dp(128)));

        TextView title = text("Petodoro", 28, true);
        col.addView(title);
        col.addView(text("A floating pet with a pomodoro timer and a stopwatch.", 16, false));

        status = text("", 15, false);
        status.setPadding(0, dp(16), 0, dp(8));
        col.addView(status);

        permissionButton = new Button(this);
        permissionButton.setText("1. Allow \"Display over other apps\"");
        permissionButton.setOnClickListener(v -> startActivity(new Intent(
                Settings.ACTION_MANAGE_OVERLAY_PERMISSION, Uri.parse("package:" + getPackageName()))));
        col.addView(permissionButton, wide());

        petButton = new Button(this);
        petButton.setOnClickListener(v -> {
            if (PetService.running) {
                stopService(new Intent(this, PetService.class));
            } else {
                startPet();
            }
            main(() -> refresh(), 300);
        });
        col.addView(petButton, wide());

        TextView help = text(
                "• Drag the pet anywhere on the screen\n"
                        + "• Tap the speech bubble to start / pause the timer\n"
                        + "• Tap the pet to pat it\n"
                        + "• Long-press the pet for its menu: choose a pet, pomodoro or stopwatch, settings\n"
                        + "• Hide it from the notification or from here",
                15, false);
        help.setGravity(Gravity.START);
        help.setPadding(0, dp(24), 0, 0);
        col.addView(help);

        ScrollView scroll = new ScrollView(this);
        scroll.addView(col);
        setContentView(scroll);

        if (Build.VERSION.SDK_INT >= 33
                && checkSelfPermission("android.permission.POST_NOTIFICATIONS") != PackageManager.PERMISSION_GRANTED) {
            requestPermissions(new String[] {"android.permission.POST_NOTIFICATIONS"}, 1);
        }
    }

    @Override
    protected void onResume() {
        super.onResume();
        // Coming back from the permission screen: show the pet right away.
        if (Settings.canDrawOverlays(this) && !PetService.running) {
            startPet();
        }
        main(this::refresh, 300);
    }

    private void startPet() {
        if (Settings.canDrawOverlays(this)) {
            startForegroundService(new Intent(this, PetService.class));
        }
    }

    private void refresh() {
        boolean allowed = Settings.canDrawOverlays(this);
        permissionButton.setEnabled(!allowed);
        petButton.setEnabled(allowed);
        petButton.setText(PetService.running ? "Hide my pet" : "2. Show my pet");
        status.setText(allowed
                ? (PetService.running ? "Your pet is on the screen. Switch to any app and it stays with you." : "Ready!")
                : "Petodoro needs permission to float above your other apps.");
    }

    private void main(Runnable r, long delayMs) {
        getWindow().getDecorView().postDelayed(r, delayMs);
    }

    private TextView text(String s, int sp, boolean bold) {
        TextView t = new TextView(this);
        t.setText(s);
        t.setTextSize(sp);
        t.setTextColor(Color.rgb(0x4a, 0x33, 0x41));
        t.setGravity(Gravity.CENTER_HORIZONTAL);
        if (bold) {
            t.setTypeface(Typeface.DEFAULT_BOLD);
        }
        return t;
    }

    private LinearLayout.LayoutParams wide() {
        LinearLayout.LayoutParams lp = new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT, LinearLayout.LayoutParams.WRAP_CONTENT);
        lp.topMargin = dp(8);
        return lp;
    }

    private int dp(int v) {
        return Math.round(v * getResources().getDisplayMetrics().density);
    }
}
