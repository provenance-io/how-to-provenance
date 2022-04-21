import { Button, Dropdown, Input } from "Components";
import { SubHeader } from "Components/Headers";
import { Name, NameList } from "Components/NameList";
import { Colors, ROOT_NAME } from "consts";
import { FunctionComponent, useState } from "react";
import { NameContractService } from "services/NameContractService";
import styled from "styled-components";

export interface NameLookupProps {

}

const Results = styled.div`

`

const SearchError = styled.div`
    background: ${Colors.WARN};
    padding: 10px;
    font-size: 1.5rem;
    margin: 10px 0;
    border-radius: 5px;
    color: ${Colors.LIGHT};
`

const NameLookupWrapper = styled.div`
    max-width: 600px;
`
export const NameLookup: FunctionComponent<NameLookupProps> = ({ }) => {
    const [value, setValue] = useState('')
    const [submitting, setSubmitting] = useState(false)
    const [lookupType, setLookupType] = useState('name')
    const valueValid = (val: string) => val.trim() != ''

    const [results, setResults] = useState<string[]>([])
    const [additionalNames, setAdditionalNames] = useState<string[]>([])
    const [resultTerm, setResultTerm] = useState('')
    const [searchError, setSearchError] = useState('')

    const nameService = new NameContractService(ROOT_NAME)

    const handleLookup = async () => {
        setSubmitting(true)
        setResults([])
        setResultTerm(value)
        setSearchError('')
        setAdditionalNames([])
        try {
            if (lookupType == 'name') {
                const address = await nameService.resolveName(value)
                setResults([address])
                const otherNames = await nameService.listNames(address)
                setAdditionalNames(otherNames.filter(name => name != value))
            } else {
                const names = await nameService.listNames(value)
                setResults(names)
            }
            setValue('')
            setSubmitting(false)
        } catch (e) {
            if (e instanceof Error) {
                setSearchError(e.message)
            }
            setSubmitting(false)
        }
    }

    return <NameLookupWrapper>
        <Dropdown label="lookup type" name="lookupType" value={lookupType} options={['select a lookup type', 'name', 'address']} onChange={(t) => setLookupType(t)}></Dropdown>
        <form>
            <Input label={lookupType.slice(0, 1).toUpperCase() + lookupType.slice(1)} value={value} onChange={(v) => setValue(v)} />
            <Button type="submit" disabled={!valueValid(value) || submitting} onClick={() => handleLookup()}>Lookup</Button>
        </form>
        {searchError && <SearchError>{searchError}</SearchError>}
        {!searchError && results.length > 0 && <Results>
            <SubHeader>Results for {resultTerm}:</SubHeader>
            <NameList>
                {results.map((res, i) => <Name key={`res-${i}`}>{res}</Name>)}
            </NameList>
        </Results>}
        {!searchError && additionalNames.length > 0 && <Results>
            <SubHeader>Additional Names:</SubHeader>
            <NameList>
                {additionalNames.map((res, i) => <Name key={`additional-${i}`}>{res}</Name>)}
            </NameList>
        </Results>}
    </NameLookupWrapper>
}
