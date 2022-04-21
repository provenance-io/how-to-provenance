import styled from 'styled-components';
import PropTypes from 'prop-types';
import { Loading } from 'Components';

const StyledButton = styled.button`
  flex-basis: ${({ width }) => width };
  ${({ width }) => width === 'auto' && 'min-width: 150px' };
  align-items: center;
  background: ${({ color }) => color };
  white-space: nowrap;
  border-radius: 6px;
  border: 1px solid ${({ color }) => color };
  text-shadow: 0 1px 0px rgba(0, 0, 0, 0.5), 0 -1px 0px rgba(0, 0, 0, 0.5), 1px 0 0px rgba(0, 0, 0, 0.5), -1px 0 0px rgba(0, 0, 0, 0.5);
  color: white;
  cursor: ${({ disabled }) => disabled ? 'not-allowed' : 'pointer' };
  display: flex;
  justify-content: center;
  letter-spacing: 0.07rem;
  transition: 250ms all;
  user-select: none;
  font-size: 1.2rem;
  height: 40px;
  padding: 0 30px;
  &:hover:not(:disabled) {
    filter: contrast(200%);
  }
  &:active:not(:disabled) {
    filter: contrast(90%);
  }
  &:disabled {
    filter: grayscale(80%);
  }
`;
const ButtonContent = styled.div``;

const Button = ({ className, color, onClick, children, disabled, width, title, type, loading }) => {
  const handleClick = () => {
    if (!disabled) {
      onClick();
    }
  }

  return (
    <StyledButton
      title={title}
      className={className}
      onClick={handleClick}
      color={color.toUpperCase()}
      onKeyPress={(e) => {
        if (e.key === 'Enter') {
          handleClick();
        }
      }}
      disabled={disabled || loading}
      width={width}
      type={type}
    >
      <ButtonContent>{loading ? <Loading /> : children}</ButtonContent>
    </StyledButton>
  );
};

Button.propTypes = {
  className: PropTypes.string,
  color: PropTypes.string,
  onClick: PropTypes.func,
  children: PropTypes.node.isRequired,
  disabled: PropTypes.bool,
  loading: PropTypes.bool,
  width: PropTypes.string,
  title: PropTypes.string,
  type: PropTypes.string,
};
Button.defaultProps = {
  className: '',
  color: '#42368E',
  onClick: () => {},
  disabled: false,
  width: 'auto',
  title: '',
  type: 'button',
  loading: false,
};

export default Button;