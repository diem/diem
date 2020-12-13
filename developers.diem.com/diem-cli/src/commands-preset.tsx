import React, { useState } from 'react';
import { CommandNames, commandsList } from '@libra-opensource/libra-client-sdk-typescript/dist/cli/commands/command';
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

interface CommandsListProps {
  onCommandSelect: (command: string) => void;
}

interface CurrentCommandProps extends CommandsListProps {
  selected: string,
}

const CurrentCommand = ({
  onCommandSelect,
  selected,
}: CurrentCommandProps) => (
  <>
    {selected === 'get_account' && <GetAccount onSubmit={onCommandSelect} />}
    {selected === 'get_transactions' && (
      <GetTransactions onSubmit={onCommandSelect} />
    )}
    {selected === 'get_transaction' && (
      <GetTransaction onSubmit={onCommandSelect} />
    )}
    {selected === 'get_account_transaction' && (
      <GetAccountTransaction onSubmit={onCommandSelect} />
    )}
    {selected === 'get_account_transactions' && (
      <GetAccountTransactions onSubmit={onCommandSelect} />
    )}
    {selected === 'get_currencies' && (
      <GetCurrencies onSubmit={onCommandSelect} />
    )}
    {selected === 'get_events' && <GetEvents onSubmit={onCommandSelect} />}
    {selected === 'get_metadata' && (
      <GetMetadata onSubmit={onCommandSelect} />
    )}
    {selected === 'submit' && <Submit onSubmit={onCommandSelect} />}
    {selected === 'address_to_bech32' && (
      <AddressToBech32 onSubmit={onCommandSelect} />
    )}
    {selected === 'address_from_bech32' && (
      <AddressFromBech32 onSubmit={onCommandSelect} />
    )}
    {selected === 'generate_keys' && (
      <GenerateKeys onSubmit={onCommandSelect} />
    )}
    {selected === 'create_account' && (
      <CreateAccount onSubmit={onCommandSelect} />
    )}
  </>
);

function CommandsPreset({ onCommandSelect }: CommandsListProps) {
  const [selected, setSelected] = useState<CommandNames>();
  return (
    <>
      <div>
        <div className="commands">
          {commandsList.map(commandName => (
            hiddenCommands.includes(commandName) ?
              '' :
              <div key={commandName}>
                <div className="command cursor-pointer">
                  <span
                    className={
                      'nav-link' + (commandName === selected ? ' active' : '')
                    }
                    onClick={(e) => {
                      e.preventDefault();
                      setSelected(
                        selected === commandName ? undefined : commandName
                      );
                    }}
                  >
                    {commandName}
                  </span>
                </div>
                {selected === commandName &&
                  <CurrentCommand selected={selected} onCommandSelect={onCommandSelect} />
                }
              </div>
          ))}
        </div>
      </div>
    </>
  );
}

export default CommandsPreset;
