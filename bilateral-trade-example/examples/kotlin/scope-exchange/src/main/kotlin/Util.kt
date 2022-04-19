import io.provenance.metadata.v1.MsgWriteScopeRequest
import io.provenance.metadata.v1.Party
import io.provenance.metadata.v1.PartyType
import io.provenance.metadata.v1.ScopeResponse

fun ScopeResponse.getChangeOwnerMessage(newOwnerAddress: String) = MsgWriteScopeRequest.newBuilder()
    .setScopeUuid(scope.scopeIdInfo.scopeUuid)
    .setSpecUuid(scope.scopeSpecIdInfo.scopeSpecUuid)
    .addSigners(scope.scope.valueOwnerAddress)
    .setScope(scope.scope.toBuilder()
        .setValueOwnerAddress(newOwnerAddress)
        .clearOwners()
        .addAllOwners(scope.scope.ownersList.filter { it.role != PartyType.PARTY_TYPE_OWNER }.plus(
            Party.newBuilder()
            .setAddress(newOwnerAddress)
            .setRole(PartyType.PARTY_TYPE_OWNER)
            .build())
        )
    )
    .build().toAny()
