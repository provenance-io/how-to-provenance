package io.p8e.demo

import io.p8e.demo.proto.LoanData
import io.provenance.scope.contract.annotations.Function
import io.provenance.scope.contract.annotations.Input
import io.provenance.scope.contract.annotations.Participants
import io.provenance.scope.contract.annotations.Record
import io.provenance.scope.contract.annotations.ScopeSpecification
import io.provenance.scope.contract.annotations.ScopeSpecificationDefinition
import io.provenance.scope.contract.proto.Specifications.PartyType.ORIGINATOR
import io.provenance.scope.contract.spec.P8eContract
import io.provenance.scope.contract.spec.P8eScopeSpecification

const val scopeNamespace = "io.p8e.demo.Loan"

@ScopeSpecificationDefinition(
    uuid = "c25680aa-32d5-4652-969f-26992222e167",
    name = scopeNamespace,
    description = "A demo loan contract",
    partiesInvolved = [ORIGINATOR]
)
class DemoLoanScopeSpecification : P8eScopeSpecification()

@Participants(roles = [ORIGINATOR])
@ScopeSpecification(names = [scopeNamespace])
open class CreateLoanScopeContract : P8eContract() {

    @Function(invokedBy = ORIGINATOR)
    @Record(name = "loan")
    open fun loan(@Input(name = "loan") loan: LoanData.Loan) = loan

    @Function(invokedBy = ORIGINATOR)
    @Record(name = "servicer")
    open fun servicer(@Input(name = "servicer") servicer: LoanData.Servicer) = servicer
}

@Participants(roles = [ORIGINATOR])
@ScopeSpecification(names = [scopeNamespace])
open class UpdateLoanScopeServicerContract(
    @Record(name = "servicer") val existingServicer: LoanData.Servicer
) : P8eContract() {

    @Function(invokedBy = ORIGINATOR)
    @Record(name = "servicer")
    open fun servicer(@Input(name = "servicer") servicer: LoanData.Servicer) = servicer
}