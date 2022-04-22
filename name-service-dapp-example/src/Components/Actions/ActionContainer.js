import styled from 'styled-components';
import PropTypes from 'prop-types';

const Container = styled.div`
  padding: 20px;
  border: 2px solid ${({ color }) => color };
  background: ${({ color, isLoading }) => isLoading ? '#CCCCCC' : `${color}50` };
  border-radius: 5px;
  display: flex;
  max-width: 100%;
  align-items: center;
  justify-content: ${({ justify }) => justify };
  ${({ noMargin }) => !noMargin && 'margin-bottom: 30px;' }
  box-shadow: 1px 1px 3px 1px ${({ color }) => color };
`;

export const ActionContainer = ({ children, color, justify, loading, noMargin }) => (
  <Container color={color} justify={justify} isLoading={loading} noMargin={noMargin}>{children}</Container>
);

ActionContainer.propTypes = {
  color: PropTypes.string,
  children: PropTypes.node,
  justify: PropTypes.string,
  loading: PropTypes.bool,
  noMargin: PropTypes.bool,
};
ActionContainer.defaultProps = {
  color: '#42368E',
  children: null,
  justify: 'center',
  loading: false,
  noMargin: false,
};
