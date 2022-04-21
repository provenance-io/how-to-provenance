import styled from "styled-components";
import PropTypes from "prop-types";
import { Colors } from "consts";

const Container = styled.div`
  margin-bottom: 20px;
  display: flex;
  flex-direction: column;
  position: relative;
`;
const Group = styled.div`
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: flex-start;
  flex-basis: 100%;
`;
const SelectContainer = styled.div`
  position: relative;
  width: 100%;
`;
const StyledSelect = styled.select`
  width: 100%;
  padding: 14px 18px;
  border-radius: 4px;
  margin: 0;
  border: 1px solid ${Colors.DARK};
  font-size: 1.4rem;
  flex-grow: ${({ type }) => (type === "radio" ? "initial" : "1")};
  font-weight: bold;
  background: ${Colors.LIGHT};
  cursor: pointer;
  color: ${Colors.DARK};
  &:focus,
  &:focus-visible,
  &:active {
    outline-color: ${Colors.DARK};
  }
  -webkit-appearance: none;
  -moz-appearance: none;
  appearance: none;
`;
const DropdownIcon = styled.div`
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  right: 18px;
  color: ${Colors.TEXT};
  pointer-events: none;
  cursor: pointer;
  font-size: 1.8rem;
  text-align: middle;
  font-weight: bold;
`;
const Label = styled.label`
  margin-bottom: 16px;
  font-weight: bold;
  font-size: 1.8rem;
  display: inline-block;
  color: #333333;
`;

const Dropdown = ({
  className,
  label,
  options,
  name,
  value: initialValue,
  onChange,
}) => {
  const renderOptions = () =>
    options.map((title, index) => (
      <option key={title} value={title} disabled={index === 0}>
        {title}
      </option>
    ));

  return (
    <Container className={className}>
      {label && <Label htmlFor={name}>{label}</Label>}
      <Group>
        <SelectContainer>
          <StyledSelect
            value={initialValue || options[0]}
            onChange={({ target }) => onChange(target.value)}
          >
            {renderOptions()}
          </StyledSelect>
          <DropdownIcon>^</DropdownIcon>
        </SelectContainer>
      </Group>
    </Container>
  );
};

Dropdown.propTypes = {
  className: PropTypes.string,
  label: PropTypes.string,
  options: PropTypes.array.isRequired,
  value: PropTypes.node,
  name: PropTypes.string.isRequired,
  onChange: PropTypes.func.isRequired,
};
Dropdown.defaultProps = {
  className: "",
  label: "",
  value: null,
};

export default Dropdown;
