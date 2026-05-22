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

rootProject.name = "NakoAndroid"
include(":app")
include(":nako-public-client-sdk")
project(":nako-public-client-sdk").projectDir = file("../../sdk/kotlin")
