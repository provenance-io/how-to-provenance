import styled from 'styled-components';
import PropTypes from 'prop-types';

const Svg = styled.svg`
  margin: auto;
  display: block;
`;

const Loading = ({ className, color, height, width }) => (
  <Svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 100 100"
    preserveAspectRatio="xMidYMid"
    className={className}
    height={height}
    width={width}
  >
    <circle
      cx="50"
      cy="50"
      fill="none"
      stroke={color}
      strokeWidth="10"
      r="35"
      strokeDasharray="164.93361431346415 56.97787143782138"
    >
        <animateTransform
          attributeName="transform"
          type="rotate"
          repeatCount="indefinite"
          dur="1s"
          values="0 50 50;360 50 50"
          keyTimes="0;1"
        />
    </circle>
</Svg>
);

Loading.propTypes = {
  className: PropTypes.string,
  color: PropTypes.string,
  height: PropTypes.string,
  width: PropTypes.string,
};
Loading.defaultProps = {
  className: '',
  color: '#eeeeee',
  height: '20px',
  width: '20px',
};

export default Loading;
