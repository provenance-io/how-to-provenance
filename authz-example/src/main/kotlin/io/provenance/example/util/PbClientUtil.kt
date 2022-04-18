package io.provenance.example.util

import io.provenance.client.grpc.GasEstimationMethod
import io.provenance.client.grpc.PbClient
import io.provenance.example.util.InputUtil.input
import io.provenance.example.util.InputUtil.inputEnum
import io.provenance.example.util.InputUtil.inputString
import java.net.URI

/**
 * A util object used to obtain a PbClient that connects to a provenance environment.  For the purposes of these examples,
 * only testnet environments (localnet included) will function with this client.  The client itself can be established
 * through this suite to connect to a mainnet, but all derived account addresses will use testnet HD pathing and be
 * derived with the prefix "tp" which will prevent transactions from being accepted by a mainnet node.
 */
object PbClientUtil {
    private const val LOCAL_CHAIN_ID = "chain-local"
    private val LOCAL_CHANNEL_URI = URI("http://localhost:9090")

    /**
     * Prompts the user for a PbClient environment by taking action based on the input PbClientType.
     * Automatically connects to localnet with provided standard environment values.
     */
    fun newClient(clientType: PbClientType? = null): PbClient {
        val messagePrefix = "[PbClient Init]"
        val derivedClientType = clientType ?: inputEnum(
            messagePrefix = messagePrefix,
            params = InputParams(
                default = DefaultParam(PbClientType.LOCAL),
            )
        )
        val (chainId, channelUri) = when (derivedClientType) {
            PbClientType.LOCAL -> LOCAL_CHAIN_ID to LOCAL_CHANNEL_URI
            PbClientType.MANUAL -> Pair(
                first = inputString("$messagePrefix Enter a chain id"),
                second = input("$messagePrefix Enter a channel URI") { URI(it) }
            )
        }
        return PbClient(
            chainId = chainId,
            channelUri = channelUri,
            gasEstimationMethod = GasEstimationMethod.MSG_FEE_CALCULATION,
        )
    }
}

/**
 * Enum switch for connecting to a Provenance node.
 */
enum class PbClientType {
    // Connects to a localnet, assumed to be created by the provenance repository's `make localnet-start` command.
    // If a localnet exists at a different location that localhost:9090, the MANUAL option must be used.
    LOCAL,
    // Allows specification of a different chain-id and channel uri for connection to a Provenance node.
    MANUAL,
}
