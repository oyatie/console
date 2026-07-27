package com.console.app.auth

import android.content.Context
import com.console.api.client.model.MobilePasskeyStepUpBinding
import com.console.api.client.model.MobilePasskeyStepUpEnvelope
import com.console.api.client.model.PasskeyStepUpAssertion
import com.console.app.data.api.ConsoleApiGateway
import kotlinx.serialization.json.JsonObject

class MobilePasskeyStepUpRepository(
    private val api: ConsoleApiGateway,
    private val credentialClient: PasskeyCredentialClient,
) {
    suspend fun requestStepUp(
        context: Context,
        binding: MobilePasskeyStepUpBinding,
    ): MobilePasskeyStepUpEnvelope {
        val start = api.startMobilePasskeyStepUp(binding)
        require(start.binding == binding) { "mobile passkey step-up binding mismatch" }

        val challengeJson = JsonObject(start.challenge).toString()
        val credential = credentialClient.getStepUpCredential(context, challengeJson)
        return MobilePasskeyStepUpEnvelope(
            binding = binding,
            assertion = PasskeyStepUpAssertion(
                ceremonyId = start.ceremonyId,
                credential = credential,
            ),
        )
    }
}