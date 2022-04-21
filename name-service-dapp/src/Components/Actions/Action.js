import { useState, useEffect } from "react";
import {
  WINDOW_MESSAGES,
  useWalletConnect,
} from "@provenanceio/walletconnect-js";
import PropTypes from "prop-types";
import { Button, Input } from "Components";
import { ActionContainer } from "./ActionContainer";

const windowMessageLookup = (windowMessage) => [
  `${WINDOW_MESSAGES[`${windowMessage}_COMPLETE`]}`,
  `${WINDOW_MESSAGES[`${windowMessage}_FAILED`]}`,
];

export const Action = ({
  method,
  setPopup,
  fields,
  buttonTxt,
  windowMessage,
}) => {
  const { walletConnectService, walletConnectState } = useWalletConnect();
  // Get loading state for specific method
  const loading = walletConnectState[`${method}Loading`];
  // Get complete and failed messages
  const [windowMsgComplete, windowMsgFailed] =
    windowMessageLookup(windowMessage);
  // Build state object from fields data (fields are an array of obj, see propTypes)
  const initialInputValues = {};
  fields.forEach(({ name, value }) => {
    initialInputValues[name] = value;
  });
  const [inputValues, setInputValues] = useState(initialInputValues);
  // Create all event listeners for this method
  useEffect(() => {
    // Delegate Hash Events
    walletConnectService.addListener(windowMsgComplete, (result) => {
      console.log(`WalletConnectJS | ${method} Complete | Result: `, result); // eslint-disable-line no-console
      setPopup(
        `${method} Complete! See console for result details`,
        "success",
        5000
      );
    });
    walletConnectService.addListener(windowMsgFailed, (result) => {
      const { error } = result;
      console.log(`WalletConnectJS | ${method} Failed | Result: `, result); // eslint-disable-line no-console
      setPopup(
        `${method} Failed! ${error} | See console for more details`,
        "failure",
        5000
      );
    });

    return () => {
      walletConnectService.removeAllListeners(windowMsgComplete);
      walletConnectService.removeAllListeners(windowMsgFailed);
    };
  }, [
    walletConnectService,
    setPopup,
    windowMsgComplete,
    windowMsgFailed,
    method,
  ]);

  const changeInputValue = (name, value) => {
    const newInputValues = { ...inputValues };
    newInputValues[name] = value;
    setInputValues(newInputValues);
  };

  const renderInputs = () =>
    fields.map(({ name, width, label, placeholder }) => (
      <Input
        key={name}
        width={width}
        value={inputValues[name]}
        label={label}
        placeholder={placeholder}
        onChange={(value) => changeInputValue(name, value)}
      />
    ));

  // If we only have a single, send the value it without the key (as itself, non obj)
  const getSendData = () =>
    Object.keys(inputValues).length > 1
      ? inputValues
      : inputValues[Object.keys(inputValues)[0]];

  return (
    <ActionContainer loading={loading}>
      {renderInputs()}
      <Button
        loading={loading}
        onClick={() => walletConnectService[method](getSendData())}
      >
        {buttonTxt}
      </Button>
    </ActionContainer>
  );
};

Action.propTypes = {
  method: PropTypes.string.isRequired,
  setPopup: PropTypes.func.isRequired,
  fields: PropTypes.arrayOf(
    PropTypes.shape({
      label: PropTypes.string,
      name: PropTypes.string,
      value: PropTypes.oneOfType([PropTypes.string, PropTypes.number]),
      placeholder: PropTypes.string,
      width: PropTypes.string,
    })
  ),
  buttonTxt: PropTypes.string.isRequired,
  windowMessage: PropTypes.string.isRequired,
};

Action.defaultProps = {
  fields: [],
};
