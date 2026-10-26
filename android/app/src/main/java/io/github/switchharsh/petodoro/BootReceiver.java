package io.github.switchharsh.petodoro;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.provider.Settings;

import java.io.File;

/** Starts the pet after a reboot if "Start on login" is on in the pet's menu. */
public class BootReceiver extends BroadcastReceiver {
    @Override
    public void onReceive(Context context, Intent intent) {
        if (!Intent.ACTION_BOOT_COMPLETED.equals(intent.getAction())) {
            return;
        }
        // Written by the Rust side (system::set_autostart) under HOME = filesDir.
        File flag = new File(context.getFilesDir(), ".config/petodoro/autostart");
        if (flag.exists() && Settings.canDrawOverlays(context)) {
            context.startForegroundService(new Intent(context, PetService.class));
        }
    }
}
