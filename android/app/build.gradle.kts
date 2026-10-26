// The Android app is a thin Java shell around the Rust library
// (libpetodoro.so in src/main/jniLibs, built by CI with `cargo rustc`).
plugins {
    id("com.android.application")
}

// Keep the app version in step with Cargo.toml.
val cargoVersion: String = Regex("""(?m)^version\s*=\s*"([^"]+)"""")
    .find(rootDir.resolve("../Cargo.toml").readText())!!.groupValues[1]
val versionParts = cargoVersion.split(".").map { it.toInt() }

android {
    namespace = "io.github.switchharsh.petodoro"
    compileSdk = 35

    defaultConfig {
        applicationId = "io.github.switchharsh.petodoro"
        minSdk = 26
        targetSdk = 35
        versionCode = versionParts[0] * 10000 + versionParts[1] * 100 + versionParts[2]
        versionName = cargoVersion
    }

    signingConfigs {
        create("release") {
            // A real key from CI secrets when available; otherwise the
            // repository's public debug key (see README).
            storeFile = file(System.getenv("PETODORO_KEYSTORE") ?: "petodoro-debug.jks")
            storePassword = System.getenv("PETODORO_KEYSTORE_PASSWORD") ?: "android"
            keyAlias = System.getenv("PETODORO_KEY_ALIAS") ?: "petodoro"
            keyPassword = System.getenv("PETODORO_KEY_PASSWORD") ?: "android"
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            signingConfig = signingConfigs.getByName("release")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
