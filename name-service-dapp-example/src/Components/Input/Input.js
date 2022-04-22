import styled from "styled-components";
import PropTypes from "prop-types";
import { Colors } from "consts";

const InputContainer = styled.div`
  position: relative;
  flex-basis: ${({ width }) => width};
  display: flex;
`;
const StyledInput = styled.input`
  padding: 4px 10px;
  height: 30px;
  width: 100%;
  box-sizing: content-box;
  border-radius: 0;
  margin-right: 4px;
  margin-bottom: 10px;
  border: 1px solid ${Colors.DARK};
`;
const Label = styled.label`
  font-size: 1rem;
  font-weight: 700;
  position: absolute;
  top: -16px;
  left: 0;
`;

const Input = ({
  className,
  label,
  value,
  onChange,
  placeholder,
  width,
  disabled,
}) => (
  <InputContainer width={width} className={className}>
    {label && <Label>{label}</Label>}
    <StyledInput
      value={value}
      placeholder={placeholder}
      onChange={({ target }) => onChange(target.value)}
      disabled={disabled}
    />
  </InputContainer>
);

Input.propTypes = {
  className: PropTypes.string,
  width: PropTypes.string,
  label: PropTypes.string,
  value: PropTypes.oneOfType([PropTypes.string, PropTypes.number]),
  onChange: PropTypes.func,
  placeholder: PropTypes.oneOfType([PropTypes.string, PropTypes.number]),
  disabled: PropTypes.bool,
};
Input.defaultProps = {
  className: "",
  width: "100%",
  label: "",
  value: "",
  onChange: () => {},
  placeholder: "Enter Value",
};

export default Input;
