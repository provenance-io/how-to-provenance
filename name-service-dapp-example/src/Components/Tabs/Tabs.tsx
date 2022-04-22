import styled from "styled-components";
import { Component, FunctionComponent, useState } from 'react';
import { Colors } from "consts";

export interface Tab {
    title: string,
    element: Component
}

export interface TabContainerProps {
    tabs: Tab[]
}

const TabContainerWrapper = styled.div`
    display: flex;
    flex-direction: column;
    margin-bottom: 20px;
`

const TabHeaderWrapper = styled.div`

`

interface TabHeaderProps {
    active?: boolean
}

const TabHeader = styled.button<TabHeaderProps>`
    border: 1px solid ${Colors.DARK};
    border-bottom: none;
    display: inline-block;
    padding: 4px;
    background: ${({ active }) => active ? Colors.DARKEN : Colors.LIGHT};
    cursor: pointer;
    &:not(:last-child) {
        border-right: none;
    }
    &:first-child {
        border-top-left-radius: 5px;
    }
    &:last-child {
        border-top-right-radius: 5px;
    }
    &:hover {
        background: ${Colors.DARKEN};
    }
`

const TabWrapper = styled.div`
    padding: 10px;
    background-color: ${Colors.LIGHT};

    border-top-right-radius: 5px;
    border-bottom-right-radius: 5px;
    border-bottom-left-radius: 5px;
    border: 1px solid ${Colors.DARK};
`

export const TabContainer: FunctionComponent<TabContainerProps> = ({ tabs }) => {
    const [selected, setSelected] = useState(0)
    return <TabContainerWrapper>
        <TabHeaderWrapper>{tabs.map((tab, i) => <TabHeader active={selected === i} key={`tab-${i}`} onClick={() => setSelected(i)}>{tab.title}</TabHeader>)}</TabHeaderWrapper>
        <TabWrapper>{tabs[selected].element}</TabWrapper>
    </TabContainerWrapper>
}
