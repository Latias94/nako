import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.gradle.api.provider.ProviderFactory
import org.gradle.api.tasks.Exec
import org.gradle.api.tasks.testing.Test
import java.util.Locale

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.kotlin.serialization)
}

android {
    namespace = "dev.taru.android"
    compileSdk = 36

    defaultConfig {
        applicationId = "dev.taru.android"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    buildTypes {
        debug {
            buildConfigField("boolean", "TARU_ALLOW_CLEARTEXT_HTTP", "true")
        }
        release {
            buildConfigField("boolean", "TARU_ALLOW_CLEARTEXT_HTTP", "false")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

val repoRoot = layout.projectDirectory.dir("../../..")
val generatedUniFfiDir = layout.buildDirectory.dir("generated/source/uniffi/main/java")
val generatedUniFfiSource = generatedUniFfiDir.map {
    it.file("uniffi/taru_client_uniffi/taru_client_uniffi.kt")
}
val generatedUniFfiJniLibsDir = layout.buildDirectory.dir("generated/jniLibs/main")
val hostUniFfiLibrary = repoRoot.file("target/debug/${hostUniFfiLibraryName()}")
val androidAbiTargets = mapOf(
    "arm64-v8a" to "aarch64-linux-android",
    "armeabi-v7a" to "armv7-linux-androideabi",
    "x86" to "i686-linux-android",
    "x86_64" to "x86_64-linux-android",
)

fun ProviderFactory.androidNdkHome(): Provider<String> =
    gradleProperty("android.ndk.home")
        .orElse(environmentVariable("ANDROID_NDK_HOME"))
        .orElse(environmentVariable("NDK_HOME"))

fun androidRustLinkerName(target: String): String =
    when (target) {
        "armv7-linux-androideabi" -> "armv7a-linux-androideabi26-clang${hostCommandExtension()}"
        else -> "${target}26-clang${hostCommandExtension()}"
    }

fun hostCommandExtension(): String =
    if (System.getProperty("os.name").lowercase(Locale.ROOT).contains("windows")) ".cmd" else ""

fun hostExecutableExtension(): String =
    if (System.getProperty("os.name").lowercase(Locale.ROOT).contains("windows")) ".exe" else ""

fun hostUniFfiLibraryName(): String {
    val osName = System.getProperty("os.name").lowercase(Locale.ROOT)
    return when {
        osName.contains("windows") -> "taru_client_uniffi.dll"
        osName.contains("mac") || osName.contains("darwin") -> "libtaru_client_uniffi.dylib"
        else -> "libtaru_client_uniffi.so"
    }
}

fun androidNdkPrebuiltName(): String {
    val osName = System.getProperty("os.name").lowercase(Locale.ROOT)
    return when {
        osName.contains("windows") -> "windows-x86_64"
        osName.contains("mac") || osName.contains("darwin") -> "darwin-x86_64"
        else -> "linux-x86_64"
    }
}

val generateTaruClientUniFfiKotlin by tasks.registering(Exec::class) {
    group = "taru rust"
    description = "Builds the host UniFFI library and generates Kotlin bindings for the Android app."
    workingDir = repoRoot.asFile
    inputs.files(
        repoRoot.file("crates/taru-client-core/src/lib.rs"),
        repoRoot.file("crates/taru-client-uniffi/src/lib.rs"),
        repoRoot.file("crates/taru-client-uniffi/Cargo.toml"),
        repoRoot.file("crates/taru-uniffi-bindgen/Cargo.toml"),
        repoRoot.file("crates/taru-uniffi-bindgen/src/main.rs"),
    )
    outputs.file(generatedUniFfiSource)
    outputs.file(hostUniFfiLibrary)

    doFirst {
        generatedUniFfiDir.get().asFile.mkdirs()
        providers.exec {
            workingDir = repoRoot.asFile
            commandLine("cargo", "build", "-p", "taru-client-uniffi")
        }.result.get().assertNormalExitValue()
    }
    commandLine(
        "cargo",
        "run",
        "-p",
        "taru-uniffi-bindgen",
        "--",
        "generate",
        "--library",
        "--no-format",
        "--language",
        "kotlin",
        "--out-dir",
        generatedUniFfiDir.get().asFile.absolutePath,
        hostUniFfiLibrary.asFile.absolutePath,
    )
}

val buildTaruClientUniFfiAndroid by tasks.registering {
    group = "taru rust"
    description = "Builds Android ABI taru-client-uniffi shared libraries for APK packaging."
    val ndkHome = providers.androidNdkHome()
    inputs.files(
        repoRoot.file("crates/taru-client-uniffi/Cargo.toml"),
        repoRoot.file("Cargo.toml"),
        repoRoot.file("Cargo.lock"),
    )
    inputs.dir(repoRoot.dir("crates/taru-client-core/src"))
    inputs.dir(repoRoot.dir("crates/taru-client-uniffi/src"))
    inputs.dir(repoRoot.dir("crates/taru-client-protocol/src"))
    androidAbiTargets.forEach { (abi, _) ->
        outputs.file(generatedUniFfiJniLibsDir.map { it.file("$abi/libtaru_client_uniffi.so") })
    }

    doLast {
        val resolvedNdkHome = ndkHome.orNull
            ?: error("Set ANDROID_NDK_HOME, NDK_HOME, or Gradle property android.ndk.home to build Taru Rust UniFFI libraries.")
        val prebuiltBin = file("$resolvedNdkHome/toolchains/llvm/prebuilt/${androidNdkPrebuiltName()}/bin")
        val ar = file("$prebuiltBin/llvm-ar${hostExecutableExtension()}")
        require(ar.isFile) { "Android NDK llvm-ar was not found at ${ar.absolutePath}" }

        androidAbiTargets.forEach { (abi, target) ->
            val linker = file("$prebuiltBin/${androidRustLinkerName(target)}")
            require(linker.isFile) { "Android NDK linker was not found at ${linker.absolutePath}" }

            providers.exec {
                workingDir = repoRoot.asFile
                val targetEnvKey = target.replace('-', '_')
                environment("AR_$targetEnvKey", ar.absolutePath)
                environment("CC_$targetEnvKey", linker.absolutePath)
                environment(
                    "CARGO_TARGET_${targetEnvKey.uppercase(Locale.ROOT)}_LINKER",
                    linker.absolutePath,
                )
                commandLine("cargo", "build", "-p", "taru-client-uniffi", "--target", target)
            }.result.get().assertNormalExitValue()

            copy {
                from(repoRoot.file("target/$target/debug/libtaru_client_uniffi.so"))
                into(generatedUniFfiJniLibsDir.map { it.dir(abi) })
            }
        }
    }
}

android.sourceSets.named("main") {
    java.srcDir(generatedUniFfiDir)
    jniLibs.srcDir(generatedUniFfiJniLibsDir)
}

tasks.named("preBuild") {
    dependsOn(generateTaruClientUniFfiKotlin)
    dependsOn(buildTaruClientUniFfiAndroid)
}

tasks.withType<Test>().configureEach {
    dependsOn(generateTaruClientUniFfiKotlin)
    systemProperty(
        "uniffi.component.taru_client_uniffi.libraryOverride",
        hostUniFfiLibrary.asFile.absolutePath,
    )
}

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_17)
    }
}

dependencies {
    implementation(project(":taru-public-client-sdk"))
    implementation(libs.androidx.activity.compose)
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.foundation)
    implementation(libs.androidx.compose.material.icons.extended)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.media3.exoplayer)
    implementation(libs.androidx.media3.exoplayer.hls)
    implementation(libs.androidx.media3.ui)
    implementation(libs.androidx.security.crypto)
    implementation(libs.coil.compose)
    implementation(libs.coil.network.okhttp)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.kotlinx.coroutines.core)
    implementation(libs.kotlinx.serialization.json)
    implementation(libs.jna)

    debugImplementation(libs.androidx.compose.ui.tooling)

    testImplementation(libs.junit)
}
