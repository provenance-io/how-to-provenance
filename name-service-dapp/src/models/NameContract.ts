import { ContractMsg } from "./ContractBase"

export class QueryNamesByAddress {
    query_names_by_address: {
        address: string
    }

    constructor(address: string) {
        this.query_names_by_address = { address }
    }
}

export interface QueryNamesByAddressResponse {
    address: string,
    names: string[]
}

export class QueryAddressByName {
    query_address_by_name: {
        name: string
    }

    constructor(name: string) {
        this.query_address_by_name = { name }
    }
}

export interface QueryAddressByNameResponse {
    address: string,
    name: string
}

export class SearchNamesRequest {
    search_for_names: {
        search: string
    }

    constructor(search: string) {
        this.search_for_names = { search };
    }
}

export class NameMetaData {
    name: string
    address: string

    constructor(name: string, address: string) {
        this.name = name;
        this.address = address;
    }
}

export interface SearchNamesQueryResponse {
    search: string,
    names: NameMetaData[]
}

export class QuerySettings {
    query_request: {} = {}
}

export interface QuerySettingsResponse {
    name: string,
    fee_amount: string,
    fee_collection_address: string,
}

export class RegisterName extends ContractMsg {
    register: {
        name: string
    } = { name: '' }

    setName(name: string): RegisterName {
        this.register.name = name
        return this
    }
}
