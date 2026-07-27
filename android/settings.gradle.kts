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

rootProject.name = "console-android"
include(":app")

includeBuild("../clients/kotlin") {
    dependencySubstitution {
        substitute(module("com.console:console-api-client")).using(project(":"))
    }
}
