# Nako Kotlin SDK

This package contains the generated Kotlin/JVM SDK foundation for Nako's Public
Client API. It is consumed through the Android Gradle build as
`:nako-public-client-sdk` and is private until publishing is designed.

## Generate

```powershell
cargo run -q -p nako-api --example emit-kotlin-sdk -- --output sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt
```

Do not edit generated source by hand.

## Check

```powershell
apps/android/gradlew.bat -p apps/android :nako-public-client-sdk:test --no-daemon
```
