import { Colors } from "consts";
import styled from "styled-components";
import {FunctionComponent, useState} from "react";
import {NameMetaData} from "models";
import {NameContractService} from "services/NameContractService";
import {ROOT_NAME} from "consts";
import { Input } from "Components";
import { SubHeader } from "Components/Headers";
import {Name, NameList} from "Components/NameList";
import {useDebouncedCallback} from "use-debounce";
import AddressLink from "Components/AddressLink";
import {BigParagraph} from "Components/Display";

export interface NameSearchProps {

}

const SearchResults = styled.div``;

const SearchError = styled.div`
    background: ${Colors.WARN};
    padding: 10px;
    font-size: 1.5rem;
    margin: 10px 0;
    border-radius: 5px;
    color: ${Colors.LIGHT};
`;

const NameSearchWrapper = styled.div`
    max-width: 600px;
`;

const isSearchValid = (val: string) => val.trim() !== '';

export const NameSearch: FunctionComponent<NameSearchProps> = ({ }) => {
    const [searchValue, setSearchValue] = useState('');
    const [searchResults, setSearchResults] = useState<NameMetaData[]>([]);
    const [searchError, setSearchError] = useState('');

    const nameService = new NameContractService(ROOT_NAME);

    const debouncedSearch = useDebouncedCallback((target: string) => {
        setSearchValue(target);
        if (!isSearchValid(target)) {
            setSearchResults([]);
            setSearchError('');
            return;
        }
        setSearchResults([]);
        setSearchError('');
        nameService.searchNames(target)
            .then(searchResultArray => {
                if (searchResultArray.length === 0) {
                    setSearchError(`No results found for "${target}"`);
                }
                setSearchResults(searchResultArray);
            })
            .catch(e => {
                if (e instanceof Error) {
                    setSearchError(e.message);
                }
            });
    }, 500);

    const handleSearchTextInput = (text: string) => {
        setSearchValue(text);
        debouncedSearch(text);
    };

    return <NameSearchWrapper>
        <form>
            <Input label={"Search for names containing"} value={searchValue} onChange={handleSearchTextInput} />
        </form>
        {searchError && <SearchError>{searchError}</SearchError>}
        {!searchError && searchResults.length > 0 && <SearchResults>
            <SubHeader>Results for "{searchValue}":</SubHeader>
            <NameList>
                {searchResults.map((result, i) => <Name key={`result-${i}`}>
                    <BigParagraph>Name: {result.name}</BigParagraph>
                    <br />
                    <AddressLink address={result.address} />
                </Name>)}
            </NameList>
        </SearchResults>}
    </NameSearchWrapper>;
};
