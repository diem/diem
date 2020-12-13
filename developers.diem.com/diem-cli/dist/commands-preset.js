import React, { useState } from 'react';
import { commandsList } from '@libra-opensource/libra-client-sdk-typescript/dist/cli/commands/command';
import GenerateKeys from './CommandsForms/GenerateKeys';
import GetAccount from './CommandsForms/GetAccount';
import GetTransaction from './CommandsForms/GetTransaction';
import GetTransactions from './CommandsForms/GetTransactions';
import GetMetadata from './CommandsForms/GetMetadata';
import GetCurrencies from './CommandsForms/GetCurrencies';
import GetAccountTransaction from './CommandsForms/GetAccountTransaction';
import GetAccountTransactions from './CommandsForms/GetAccountTransactions';
import GetEvents from './CommandsForms/GetEvents';
import Submit from './CommandsForms/Submit';
import AddressToBech32 from './CommandsForms/AddressToBech32';
import AddressFromBech32 from './CommandsForms/AddressFromBech32';
import CreateAccount from './CommandsForms/CreateAccount';
const hiddenCommands = ['submit'];
const CurrentCommand = ({ onCommandSelect, selected, }) => (React.createElement(React.Fragment, null,
    selected === 'get_account' && React.createElement(GetAccount, { onSubmit: onCommandSelect }),
    selected === 'get_transactions' && (React.createElement(GetTransactions, { onSubmit: onCommandSelect })),
    selected === 'get_transaction' && (React.createElement(GetTransaction, { onSubmit: onCommandSelect })),
    selected === 'get_account_transaction' && (React.createElement(GetAccountTransaction, { onSubmit: onCommandSelect })),
    selected === 'get_account_transactions' && (React.createElement(GetAccountTransactions, { onSubmit: onCommandSelect })),
    selected === 'get_currencies' && (React.createElement(GetCurrencies, { onSubmit: onCommandSelect })),
    selected === 'get_events' && React.createElement(GetEvents, { onSubmit: onCommandSelect }),
    selected === 'get_metadata' && (React.createElement(GetMetadata, { onSubmit: onCommandSelect })),
    selected === 'submit' && React.createElement(Submit, { onSubmit: onCommandSelect }),
    selected === 'address_to_bech32' && (React.createElement(AddressToBech32, { onSubmit: onCommandSelect })),
    selected === 'address_from_bech32' && (React.createElement(AddressFromBech32, { onSubmit: onCommandSelect })),
    selected === 'generate_keys' && (React.createElement(GenerateKeys, { onSubmit: onCommandSelect })),
    selected === 'create_account' && (React.createElement(CreateAccount, { onSubmit: onCommandSelect }))));
function CommandsPreset({ onCommandSelect }) {
    const [selected, setSelected] = useState();
    return (React.createElement(React.Fragment, null,
        React.createElement("div", null,
            React.createElement("div", { className: "commands" }, commandsList.map(commandName => (hiddenCommands.includes(commandName) ?
                '' :
                React.createElement("div", { key: commandName },
                    React.createElement("div", { className: "command cursor-pointer" },
                        React.createElement("span", { className: 'nav-link' + (commandName === selected ? ' active' : ''), onClick: (e) => {
                                e.preventDefault();
                                setSelected(selected === commandName ? undefined : commandName);
                            } }, commandName)),
                    selected === commandName &&
                        React.createElement(CurrentCommand, { selected: selected, onCommandSelect: onCommandSelect }))))))));
}
export default CommandsPreset;
