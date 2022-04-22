import { Colors } from "consts";
import styled from "styled-components";

export const NameList = styled.ul`
list-style-type: none;
padding-left: 0;
border-radius: 5px;
border: 1px solid ${Colors.DARKEN};
overflow: hidden;
`;
export const Name = styled.li`
font-weight: 600;
font-size: 1.2rem;
padding: 10px;
background: white;
&:not(:last-child) {
  border-bottom: 1px solid ${Colors.DARKEN};
}
`;