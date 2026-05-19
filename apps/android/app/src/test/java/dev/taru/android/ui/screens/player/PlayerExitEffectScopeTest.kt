package dev.taru.android.ui.screens.player

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineName
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PlayerExitEffectScopeTest {
    @Test
    fun playerExitEffectsLaunchOnInjectedScope() = runBlocking {
        val parent = Job()
        val scope = CoroutineScope(parent + Dispatchers.Unconfined + CoroutineName("player-exit-scope"))
        val observedName = CompletableDeferred<String?>()
        val release = CompletableDeferred<Unit>()

        val job = launchPlayerExitEffect(scope) {
            observedName.complete(currentCoroutineContext()[CoroutineName]?.name)
            release.await()
        }

        assertEquals("player-exit-scope", observedName.await())
        assertTrue(job.isActive)
        release.complete(Unit)
        job.join()
        assertEquals(true, job.isCompleted)
    }

    @Test
    fun playerExitEffectsDoNotRunWhenInjectedScopeIsCancelled() {
        val parent = Job()
        val scope = CoroutineScope(parent + Dispatchers.Unconfined)
        scope.cancel()
        var ran = false

        val job = launchPlayerExitEffect(scope) {
            ran = true
        }

        assertFalse(ran)
        assertTrue(job.isCancelled)
    }
}
