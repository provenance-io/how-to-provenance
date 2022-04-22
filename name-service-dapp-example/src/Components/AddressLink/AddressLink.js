import styled from "styled-components";
import PropTypes from "prop-types";
import {EXPLORER_URL} from "consts";

const Text = styled.p`
  font-size: 1.6rem;
  line-height: 3rem;
  margin: 0;
`;

const AddressLink = ({
    className,
    address,
}) => {
    return <Text className={className}>
        Address:{" "}
        <a
            href={`${EXPLORER_URL}/accounts/${address}`}
            target="_blank"
            rel="noreferrer"
        >
            {address}
        </a>
    </Text>;
};

AddressLink.propTypes = {
    className: PropTypes.string,
    address: PropTypes.string.isRequired,
};

AddressLink.defaultProps = {
    className: "",
};

export default AddressLink;
