import {
    NameMetaData,
    QueryAddressByName,
    QueryAddressByNameResponse,
    QueryNamesByAddress,
    QueryNamesByAddressResponse,
    QuerySettingsResponse,
    SearchNamesQueryResponse,
    SearchNamesRequest,
} from 'models';
import { WasmService } from 'services';
import { MsgExecuteContract } from '@provenanceio/wallet-lib/lib/proto/cosmwasm/wasm/v1/tx_pb'
import { Coin } from '@provenanceio/wallet-lib/lib/proto/cosmos/base/v1beta1/coin_pb'
import { Any } from '@provenanceio/wallet-lib/lib/proto/google/protobuf/any_pb'
import { QuerySettings, RegisterName } from '../models/NameContract';
import { FEE_DENOM } from 'consts';

export class NameContractService {
    wasmService = new WasmService()
    contractAddress: string | null = null
    rootName: string

    constructor(rootName: string) {
        this.rootName = rootName
    }

    async getContractAddress(): Promise<string> {
        if (this.contractAddress != null) {
            return this.contractAddress
        }
        this.contractAddress = await this.wasmService.lookupContractByName(this.rootName)
        return this.contractAddress
    }

    async getContractConfig(): Promise<QuerySettingsResponse> {
        return this.wasmService.queryWasmCustom<QuerySettings, QuerySettingsResponse>(await this.getContractAddress(), new QuerySettings())
    }

    async listNames(address: string): Promise<string[]> {
        const queryRes = await this.wasmService.queryWasmCustom<QueryNamesByAddress, QueryNamesByAddressResponse>(await this.getContractAddress(), new QueryNamesByAddress(address))
        
        return queryRes.names
    }

    async resolveName(name: string): Promise<string> {
        const queryRes = await this.wasmService.queryWasmCustom<QueryAddressByName, QueryAddressByNameResponse>(await this.getContractAddress(), new QueryAddressByName(name))
        
        return queryRes.address
    }

    async searchNames(search: string): Promise<NameMetaData[]> {
        const queryRes = await this.wasmService.queryWasmCustom<SearchNamesRequest, SearchNamesQueryResponse>(await this.getContractAddress(), new SearchNamesRequest(search));

        return queryRes.names;
    }

    async generateNameRegisterBase64Message(name: string, address: string): Promise<string> {
        const [contractAddr, contractConfig] = await Promise.all([
            this.getContractAddress(),
            this.getContractConfig()
        ])
        
        const message = new MsgExecuteContract()
            .setMsg(Buffer.from(new RegisterName().setName(name).toJson(), 'utf-8').toString('base64'))
            .setFundsList([new Coin().setAmount(contractConfig.fee_amount).setDenom(FEE_DENOM)])
            .setContract(contractAddr)
            .setSender(address);
        // Directly hardcoded from https://github.com/CuCreekCo/ProvenanceWalletConnect/blob/d2227d716ddb3f95783624b50e0e70220e33a858/ProvenanceWalletConnect/Handlers/WalletConnectHandlers.swift#L408
        const any = new Any()
            .setTypeUrl("/cosmwasm.wasm.v1.MsgExecuteContract")
            .setValue(message.serializeBinary());
        return Buffer.from(any.serializeBinary()).toString("base64");
    }
}
