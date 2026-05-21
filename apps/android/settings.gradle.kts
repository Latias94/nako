pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "TaruAndroid"
include(":app")
include(":taru-public-client-sdk")
project(":taru-public-client-sdk").projectDir = file("../../sdk/kotlin")
