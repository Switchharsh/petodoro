package io.github.switchharsh.petodoro;

import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.app.Service;
import android.content.Intent;
import android.content.pm.ServiceInfo;
import android.content.res.Configuration;
import android.graphics.Bitmap;
import android.graphics.PixelFormat;
import android.graphics.Rect;
import android.media.Ringtone;
import android.media.RingtoneManager;
import android.os.Build;
import android.os.Handler;
import android.os.IBinder;
import android.os.Looper;
import android.util.DisplayMetrics;
import android.view.Gravity;
import android.view.HapticFeedbackConstants;
import android.view.MotionEvent;
import android.view.View;
import android.view.ViewConfiguration;
import android.view.WindowManager;
import android.widget.ImageView;

import java.nio.ByteBuffer;
import java.util.HashMap;

/**
 * Keeps the pet alive while you use other apps. The pet itself (timers,
 * drawing, menu) is the Rust library; this service only owns the overlay
 * windows it asks for and forwards touches back to it.
 */
public class PetService extends Service {
    static {
        System.loadLibrary("petodoro");
    }

    static volatile boolean running;

    static final String ACTION_STOP = "io.github.switchharsh.petodoro.STOP";
    private static final String CHANNEL_PET = "pet";
    private static final String CHANNEL_TIMER = "timer";
    private static final int ONGOING_ID = 1;

    private final Handler main = new Handler(Looper.getMainLooper());
    private final HashMap<Integer, Win> windows = new HashMap<>();
    private WindowManager wm;
    private int timerNotificationId = 100;

    private static class Win {
        ImageView view;
        WindowManager.LayoutParams lp;
        Bitmap bitmap;
    }

    native void nativeStart(String filesDir, int width, int height, float density);

    native void nativeTouch(int id, int action, int x, int y, int rawX, int rawY);

    native void nativeScreen(int width, int height);

    native void nativeStop();

    @Override
    public void onCreate() {
        super.onCreate();
        wm = getSystemService(WindowManager.class);
        createChannels();
        startForegroundCompat();
        running = true;
        int[] s = screenSize();
        nativeStart(getFilesDir().getAbsolutePath(), s[0], s[1], getResources().getDisplayMetrics().density);
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        if (intent != null && ACTION_STOP.equals(intent.getAction())) {
            stopSelf();
            return START_NOT_STICKY;
        }
        return START_STICKY;
    }

    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }

    @Override
    public void onDestroy() {
        running = false;
        nativeStop();
        for (Win w : windows.values()) {
            removeView(w.view);
        }
        windows.clear();
        super.onDestroy();
    }

    @Override
    public void onConfigurationChanged(Configuration config) {
        super.onConfigurationChanged(config);
        int[] s = screenSize();
        nativeScreen(s[0], s[1]);
    }

    private int[] screenSize() {
        if (Build.VERSION.SDK_INT >= 30) {
            Rect b = wm.getCurrentWindowMetrics().getBounds();
            return new int[] {b.width(), b.height()};
        }
        DisplayMetrics m = new DisplayMetrics();
        wm.getDefaultDisplay().getRealMetrics(m);
        return new int[] {m.widthPixels, m.heightPixels};
    }

    // ------------------------------------------------------- notifications

    private void createChannels() {
        NotificationManager nm = getSystemService(NotificationManager.class);
        NotificationChannel pet = new NotificationChannel(CHANNEL_PET, "Pet is running", NotificationManager.IMPORTANCE_MIN);
        pet.setDescription("Shown while your pet is on screen");
        nm.createNotificationChannel(pet);
        NotificationChannel timer = new NotificationChannel(CHANNEL_TIMER, "Timer", NotificationManager.IMPORTANCE_HIGH);
        timer.setDescription("Focus session and break finished");
        nm.createNotificationChannel(timer);
    }

    private void startForegroundCompat() {
        PendingIntent open = PendingIntent.getActivity(
                this, 0, new Intent(this, MainActivity.class), PendingIntent.FLAG_IMMUTABLE);
        PendingIntent stop = PendingIntent.getService(
                this, 1, new Intent(this, PetService.class).setAction(ACTION_STOP), PendingIntent.FLAG_IMMUTABLE);
        Notification n = new Notification.Builder(this, CHANNEL_PET)
                .setSmallIcon(android.R.drawable.ic_popup_reminder)
                .setContentTitle("Your pet is keeping you company")
                .setContentText("Long-press the pet for its menu")
                .setContentIntent(open)
                .addAction(new Notification.Action.Builder(null, "Hide pet", stop).build())
                .setOngoing(true)
                .build();
        if (Build.VERSION.SDK_INT >= 34) {
            startForeground(ONGOING_ID, n, ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE);
        } else {
            startForeground(ONGOING_ID, n);
        }
    }

    // ------------------------------------------ called from Rust (pet thread)

    public void showNotification(String title, String body) {
        main.post(() -> {
            Notification n = new Notification.Builder(this, CHANNEL_TIMER)
                    .setSmallIcon(android.R.drawable.ic_popup_reminder)
                    .setContentTitle(title)
                    .setContentText(body)
                    .setStyle(new Notification.BigTextStyle().bigText(body))
                    .setAutoCancel(true)
                    .build();
            getSystemService(NotificationManager.class).notify(timerNotificationId++, n);
        });
    }

    public void chime() {
        main.post(() -> {
            Ringtone r = RingtoneManager.getRingtone(this, RingtoneManager.getDefaultUri(RingtoneManager.TYPE_NOTIFICATION));
            if (r != null) {
                r.play();
            }
        });
    }

    public void quit() {
        main.post(this::stopSelf);
    }

    public void createWindow(int id, int x, int y, int w, int h, boolean menu) {
        main.post(() -> {
            int flags = WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE
                    | WindowManager.LayoutParams.FLAG_LAYOUT_NO_LIMITS
                    | WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN;
            if (menu) {
                // Tapping anywhere else closes the menu.
                flags |= WindowManager.LayoutParams.FLAG_WATCH_OUTSIDE_TOUCH;
            }
            WindowManager.LayoutParams lp = new WindowManager.LayoutParams(
                    w, h, WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY, flags, PixelFormat.TRANSLUCENT);
            lp.gravity = Gravity.TOP | Gravity.START;
            lp.x = x;
            lp.y = y;
            if (Build.VERSION.SDK_INT >= 28) {
                lp.layoutInDisplayCutoutMode = WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES;
            }
            Win win = new Win();
            win.view = new ImageView(this);
            win.view.setScaleType(ImageView.ScaleType.FIT_XY);
            win.lp = lp;
            attachTouch(id, win.view, menu);
            try {
                wm.addView(win.view, lp);
                windows.put(id, win);
            } catch (RuntimeException e) {
                // No overlay permission: nothing to show.
                stopSelf();
            }
        });
    }

    public void moveWindow(int id, int x, int y) {
        main.post(() -> {
            Win w = windows.get(id);
            if (w != null) {
                w.lp.x = x;
                w.lp.y = y;
                wm.updateViewLayout(w.view, w.lp);
            }
        });
    }

    public void resizeWindow(int id, int x, int y, int width, int height) {
        main.post(() -> {
            Win w = windows.get(id);
            if (w != null) {
                w.lp.x = x;
                w.lp.y = y;
                w.lp.width = width;
                w.lp.height = height;
                wm.updateViewLayout(w.view, w.lp);
            }
        });
    }

    public void destroyWindow(int id) {
        main.post(() -> {
            Win w = windows.remove(id);
            if (w != null) {
                removeView(w.view);
            }
        });
    }

    /** Pixels are premultiplied RGBA, exactly ARGB_8888's memory layout. */
    public void present(int id, int width, int height, byte[] pixels) {
        main.post(() -> {
            Win w = windows.get(id);
            if (w == null) {
                return;
            }
            if (w.bitmap == null || w.bitmap.getWidth() != width || w.bitmap.getHeight() != height) {
                w.bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888);
            }
            w.bitmap.copyPixelsFromBuffer(ByteBuffer.wrap(pixels));
            w.view.setImageBitmap(w.bitmap);
            w.view.invalidate();
        });
    }

    private void removeView(View v) {
        try {
            wm.removeView(v);
        } catch (RuntimeException ignored) {
            // Already gone.
        }
    }

    // ------------------------------------------------------------- touch

    /** Touch to mouse: drag = move, tap = click, long press = right click. */
    private void attachTouch(int id, ImageView view, boolean menu) {
        final float slop = ViewConfiguration.get(this).getScaledTouchSlop();
        final float[] down = new float[2];
        final int[] last = new int[4];
        final boolean[] longFired = {false};
        final Runnable longPress = () -> {
            longFired[0] = true;
            view.performHapticFeedback(HapticFeedbackConstants.LONG_PRESS);
            nativeTouch(id, 3, last[0], last[1], last[2], last[3]);
        };
        view.setOnTouchListener((v, e) -> {
            last[0] = (int) e.getX();
            last[1] = (int) e.getY();
            last[2] = (int) e.getRawX();
            last[3] = (int) e.getRawY();
            switch (e.getActionMasked()) {
                case MotionEvent.ACTION_DOWN:
                    longFired[0] = false;
                    down[0] = e.getRawX();
                    down[1] = e.getRawY();
                    if (!menu) {
                        main.postDelayed(longPress, ViewConfiguration.getLongPressTimeout());
                    }
                    nativeTouch(id, 0, last[0], last[1], last[2], last[3]);
                    break;
                case MotionEvent.ACTION_MOVE:
                    if (longFired[0]) {
                        break;
                    }
                    if (Math.hypot(e.getRawX() - down[0], e.getRawY() - down[1]) > slop) {
                        main.removeCallbacks(longPress);
                    }
                    nativeTouch(id, 2, last[0], last[1], last[2], last[3]);
                    break;
                case MotionEvent.ACTION_UP:
                case MotionEvent.ACTION_CANCEL:
                    main.removeCallbacks(longPress);
                    if (!longFired[0]) {
                        nativeTouch(id, 1, last[0], last[1], last[2], last[3]);
                    }
                    break;
                case MotionEvent.ACTION_OUTSIDE:
                    nativeTouch(id, 4, 0, 0, 0, 0);
                    break;
                default:
                    break;
            }
            return true;
        });
    }
}
