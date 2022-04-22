import { useEffect } from 'react';
import { WINDOW_MESSAGES } from '@provenanceio/walletconnect-js';
import styled from 'styled-components';
import PropTypes from 'prop-types';
import { Button } from 'Components';
import { ActionContainer } from './ActionContainer';

const Info = styled.div`
  font-size: 1.4rem;
  margin-right: 40px;
`;
export const Connect = ({ walletConnectService, setPopup }) => {
  const color = '#498AFD';

  useEffect(() => {
    walletConnectService.addListener(WINDOW_MESSAGES.CONNECTED, (result) => {
      console.log('WalletConnectJS | Connected | Result: ', result); // eslint-disable-line no-console
      setPopup('Wallet Connected', 'success');
    });

    return () => {
      walletConnectService.removeAllListeners(WINDOW_MESSAGES.CONNECTED);
    }
  }, [walletConnectService, setPopup]);

  return (
    <ActionContainer color={color} noMargin>
      <Info>Connect Wallet</Info>
      <Button width="20%" color={color} onClick={walletConnectService.connect}>Connect</Button>
    </ActionContainer>
  );
};

Connect.propTypes = {
  walletConnectService: PropTypes.shape({
    connect: PropTypes.func,
    addListener: PropTypes.func,
    removeAllListeners: PropTypes.func,
  }).isRequired,
  setPopup: PropTypes.func.isRequired,
};
