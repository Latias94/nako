# Taru Kotlin SDK

This package contains the generated Kotlin/JVM SDK foundation for Taru's Public
Client API. It is consumed through the Android Gradle build as
`:taru-public-client-sdk` and is private until publishing is designed.

## Generate

```powershell
cargo run -q -p taru-api --example emit-kotlin-sdk -- --output sdk/kotlin/src/main/kotlin/dev/taru/sdk/TaruClientSdk.kt
```

Do not edit generated source by hand.

## Check

```powershell
apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon
```
