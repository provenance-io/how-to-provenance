package io.provenance.example.util

import io.provenance.client.grpc.GasEstimationMethod
import io.provenance.client.grpc.PbClient
import java.net.URI

object PbClientUtil {
    private const val LOCAL_CHAIN_ID = "chain-local"
    private val LOCAL_CHANNEL_URI = URI("http://localhost:9090")

    fun newClient(): PbClient {
        val messagePrefix = "[PbClient Init]"
        val (chainId, channelUri) = when (captureEnumOrNull<PbClientType>(messagePrefix)) {
            PbClientType.LOCAL -> LOCAL_CHAIN_ID to LOCAL_CHANNEL_URI
            PbClientType.MANUAL -> Pair(
                first = captureString("$messagePrefix Enter a chain id:"),
                second = captureInput("$messagePrefix Enter a channel URI:") { URI(it) }
            )
        }
        return PbClient(
            chainId = chainId,
            channelUri = channelUri,
            gasEstimationMethod = GasEstimationMethod.MSG_FEE_CALCULATION,
        )
    }
}

enum class PbClientType {
    LOCAL,
    MANUAL,
}
