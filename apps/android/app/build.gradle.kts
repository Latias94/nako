import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.gradle.api.provider.ProviderFactory
import org.gradle.api.tasks.Exec
import org.gradle.api.tasks.testing.Test
import java.util.Locale
import java.util.zip.ZipFile

val androidAbiTargets = mapOf(
    "arm64-v8a" to "aarch64-linux-android",
    "armeabi-v7a" to "armv7-linux-androideabi",
    "x86" to "i686-linux-android",
    "x86_64" to "x86_64-linux-android",
)
val selectedAndroidAbiTargets = selectedAndroidAbiTargets()

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.kotlin.serialization)
}

android {
    namespace = "dev.nako.android"
    compileSdk = 36

    defaultConfig {
        applicationId = "dev.nako.android"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        ndk {
            abiFilters += selectedAndroidAbiTargets.keys
        }
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    buildTypes {
        debug {
            buildConfigField("boolean", "NAKO_ALLOW_CLEARTEXT_HTTP", "true")
        }
        release {
            buildConfigField("boolean", "NAKO_ALLOW_CLEARTEXT_HTTP", "false")
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
    it.file("uniffi/nako_client_uniffi/nako_client_uniffi.kt")
}
val generatedUniFfiDebugJniLibsDir = layout.buildDirectory.dir("generated/jniLibs/debug")
val generatedUniFfiReleaseJniLibsDir = layout.buildDirectory.dir("generated/jniLibs/release")
val generatedHostJnaResourcesDir = layout.buildDirectory.dir("generated/resources/jna-host")
val hostUniFfiLibrary = repoRoot.file("target/debug/${hostUniFfiLibraryName()}")
val jnaHostDispatch by configurations.creating {
    isCanBeConsumed = false
    isCanBeResolved = true
}

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
        osName.contains("windows") -> "nako_client_uniffi.dll"
        osName.contains("mac") || osName.contains("darwin") -> "libnako_client_uniffi.dylib"
        else -> "libnako_client_uniffi.so"
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

fun selectedAndroidAbiTargets(): Map<String, String> {
    val requestedAbis = providers.gradleProperty("nakoRustAndroidAbis")
        .orElse(providers.gradleProperty("nako.rust.android.abis"))
        .orNull
        ?.split(',', ';', ' ')
        ?.map { it.trim() }
        ?.filter { it.isNotEmpty() }
        ?.distinct()
        ?: androidAbiTargets.keys.toList()
    val unknownAbis = requestedAbis.filterNot(androidAbiTargets::containsKey)
    require(unknownAbis.isEmpty()) {
        "Unknown nako.rust.android.abis value(s): ${unknownAbis.joinToString()}. " +
            "Supported values: ${androidAbiTargets.keys.joinToString()}."
    }
    return requestedAbis.associateWith { abi -> androidAbiTargets.getValue(abi) }
}

val buildNakoClientUniFfiHost by tasks.registering(Exec::class) {
    group = "nako rust"
    description = "Builds the host nako-client-uniffi shared library for JVM tests and binding generation."
    workingDir = repoRoot.asFile
    inputs.files(
        repoRoot.file("crates/nako-client-uniffi/Cargo.toml"),
        repoRoot.file("Cargo.toml"),
        repoRoot.file("Cargo.lock"),
    )
    inputs.dir(repoRoot.dir("crates/nako-client-core/src"))
    inputs.dir(repoRoot.dir("crates/nako-client-protocol/src"))
    inputs.dir(repoRoot.dir("crates/nako-client-uniffi/src"))
    outputs.file(hostUniFfiLibrary)
    commandLine("cargo", "build", "-p", "nako-client-uniffi")
}

val generateNakoClientUniFfiKotlin by tasks.registering(Exec::class) {
    group = "nako rust"
    description = "Generates Kotlin bindings for the Android app from the host UniFFI library."
    workingDir = repoRoot.asFile
    dependsOn(buildNakoClientUniFfiHost)
    inputs.files(
        repoRoot.file("crates/nako-client-core/src/lib.rs"),
        repoRoot.file("crates/nako-client-uniffi/src/lib.rs"),
        repoRoot.file("crates/nako-client-uniffi/Cargo.toml"),
        repoRoot.file("crates/nako-uniffi-bindgen/Cargo.toml"),
        repoRoot.file("crates/nako-uniffi-bindgen/src/main.rs"),
    )
    inputs.file(hostUniFfiLibrary)
    outputs.file(generatedUniFfiSource)

    doFirst {
        generatedUniFfiDir.get().asFile.mkdirs()
    }
    commandLine(
        "cargo",
        "run",
        "-p",
        "nako-uniffi-bindgen",
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

fun registerBuildNakoClientUniFfiAndroidTask(
    variantName: String,
    rustProfile: String,
    outputDir: Provider<Directory>,
) = tasks.register("buildNakoClientUniFfi${variantName.replaceFirstChar { it.uppercase() }}Android") {
    group = "nako rust"
    description = "Builds $variantName Android ABI nako-client-uniffi shared libraries for APK packaging."
    val ndkHome = providers.androidNdkHome()
    inputs.files(
        repoRoot.file("crates/nako-client-uniffi/Cargo.toml"),
        repoRoot.file("Cargo.toml"),
        repoRoot.file("Cargo.lock"),
    )
    inputs.dir(repoRoot.dir("crates/nako-client-core/src"))
    inputs.dir(repoRoot.dir("crates/nako-client-uniffi/src"))
    inputs.dir(repoRoot.dir("crates/nako-client-protocol/src"))
    inputs.property("nakoRustAndroidAbis", selectedAndroidAbiTargets.keys.joinToString(","))
    outputs.dir(outputDir)

    doLast {
        val resolvedNdkHome = ndkHome.orNull
            ?: error("Set ANDROID_NDK_HOME, NDK_HOME, or Gradle property android.ndk.home to build Nako Rust UniFFI libraries.")
        val prebuiltBin = file("$resolvedNdkHome/toolchains/llvm/prebuilt/${androidNdkPrebuiltName()}/bin")
        val ar = file("$prebuiltBin/llvm-ar${hostExecutableExtension()}")
        require(ar.isFile) { "Android NDK llvm-ar was not found at ${ar.absolutePath}" }
        val outputRoot = outputDir.get().asFile
        outputRoot.deleteRecursively()
        outputRoot.mkdirs()

        selectedAndroidAbiTargets.forEach { (abi, target) ->
            val linker = file("$prebuiltBin/${androidRustLinkerName(target)}")
            require(linker.isFile) { "Android NDK linker was not found at ${linker.absolutePath}" }

            providers.exec {
                workingDir = repoRoot.asFile
                val targetEnvKey = target.replace('-', '_')
                val cargoCommand = mutableListOf(
                    "cargo",
                    "build",
                    "-p",
                    "nako-client-uniffi",
                    "--target",
                    target,
                )
                if (rustProfile == "release") {
                    cargoCommand += "--release"
                }
                environment("AR_$targetEnvKey", ar.absolutePath)
                environment("CC_$targetEnvKey", linker.absolutePath)
                environment(
                    "CARGO_TARGET_${targetEnvKey.uppercase(Locale.ROOT)}_LINKER",
                    linker.absolutePath,
                )
                commandLine(cargoCommand)
            }.result.get().assertNormalExitValue()

            copy {
                from(repoRoot.file("target/$target/$rustProfile/libnako_client_uniffi.so"))
                into(outputDir.map { it.dir(abi) })
            }
        }
    }
}

android.sourceSets.named("main") {
    java.srcDir(generatedUniFfiDir)
}

android.sourceSets.named("debug") {
    jniLibs.srcDir(generatedUniFfiDebugJniLibsDir)
}

android.sourceSets.named("release") {
    jniLibs.srcDir(generatedUniFfiReleaseJniLibsDir)
}

val buildNakoClientUniFfiDebugAndroid = registerBuildNakoClientUniFfiAndroidTask(
    variantName = "debug",
    rustProfile = "debug",
    outputDir = generatedUniFfiDebugJniLibsDir,
)
val buildNakoClientUniFfiReleaseAndroid = registerBuildNakoClientUniFfiAndroidTask(
    variantName = "release",
    rustProfile = "release",
    outputDir = generatedUniFfiReleaseJniLibsDir,
)

val extractHostJnaDispatch by tasks.registering {
    group = "nako rust"
    description = "Extracts host JNA native dispatch resources for JVM tests."
    val outputDir = generatedHostJnaResourcesDir
    inputs.files(jnaHostDispatch)
    outputs.dir(outputDir)

    doLast {
        val hostResourceDir = when {
            System.getProperty("os.name").lowercase(Locale.ROOT).contains("windows") -> "win32-x86-64"
            System.getProperty("os.name").lowercase(Locale.ROOT).contains("mac") ||
                System.getProperty("os.name").lowercase(Locale.ROOT).contains("darwin") -> "darwin-aarch64"
            else -> "linux-x86-64"
        }
        val outputRoot = outputDir.get().asFile
        outputRoot.deleteRecursively()
        outputRoot.mkdirs()

        val jnaArtifact = jnaHostDispatch.files
            .firstOrNull { it.name.startsWith("jna-") && it.extension == "jar" }
            ?: error("JNA test runtime jar was not found.")
        ZipFile(jnaArtifact).use { zip ->
            zip.entries().asSequence()
                .filter { entry ->
                    !entry.isDirectory &&
                        entry.name.startsWith("com/sun/jna/$hostResourceDir/") &&
                        entry.name.substringAfterLast('/').contains("jnidispatch")
                }
                .forEach { entry ->
                    val target = outputRoot.resolve(entry.name)
                    target.parentFile.mkdirs()
                    zip.getInputStream(entry).use { input ->
                        target.outputStream().use { output -> input.copyTo(output) }
                    }
                }
        }
    }
}

tasks.matching { it.name.endsWith("Kotlin") && it.name.startsWith("compile") }.configureEach {
    dependsOn(generateNakoClientUniFfiKotlin)
}

tasks.matching { it.name == "mergeDebugJniLibFolders" }.configureEach {
    dependsOn(buildNakoClientUniFfiDebugAndroid)
}

tasks.matching { it.name == "mergeReleaseJniLibFolders" }.configureEach {
    dependsOn(buildNakoClientUniFfiReleaseAndroid)
}

tasks.withType<Test>().configureEach {
    dependsOn(generateNakoClientUniFfiKotlin)
    dependsOn(extractHostJnaDispatch)
    doFirst {
        classpath += files(generatedHostJnaResourcesDir)
    }
    systemProperty(
        "uniffi.component.nako_client_uniffi.libraryOverride",
        hostUniFfiLibrary.asFile.absolutePath,
    )
}

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_17)
    }
}

dependencies {
    implementation(project(":nako-public-client-sdk"))
    implementation(libs.androidx.activity.compose)
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.foundation)
    implementation(libs.androidx.compose.material.icons.extended)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.androidx.media3.exoplayer)
    implementation(libs.androidx.media3.exoplayer.hls)
    implementation(libs.androidx.media3.ui)
    implementation(libs.coil.compose)
    implementation(libs.coil.network.okhttp)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.kotlinx.coroutines.core)
    implementation(libs.kotlinx.serialization.json)
    implementation(libs.jna) {
        artifact {
            type = "aar"
        }
    }
    jnaHostDispatch(libs.jna)

    debugImplementation(libs.androidx.compose.ui.tooling)

    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.test.ext.junit)
    androidTestImplementation(libs.androidx.test.runner)
}
