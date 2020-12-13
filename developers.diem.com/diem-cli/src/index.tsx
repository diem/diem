import * as React from 'react';
import { useEffect } from 'react';
import { Terminal } from 'xterm-dfuzr';
import LocalEchoController from 'local-echo';
import { FitAddon } from 'xterm-addon-fit';
import yargsParser from 'yargs-parser';
import 'xterm/css/xterm.css';
import CommandsPreset from './commands-preset';
import * as cli from '@libra-opensource/libra-client-sdk-typescript/dist/cli';
import {
  CommandNames,
  CommandsArgs,
  commandsList
} from '@libra-opensource/libra-client-sdk-typescript/dist/cli/commands/command';
import JSONBigint from 'json-bigint';
import Transcript from './transcript';

const JSONbigNative = JSONBigint({
  useNativeBigInt: true
});

const MANUAL_EXECUTION = 'MANUAL_EXECUTION';

export default function DiemTerminal({className}: {className: string}) {
  const PROMPT = '≋Diem> ';
  const GREETINGS = [
    'Welcome to Diem Shell 💁‍♀️',
    'This is a fully local Diem testnet client, running entirely on your browser!',
    ''
  ];
  const isLive = true;

  const terminalRef = React.createRef<HTMLDivElement>();
  const __terminal = new Terminal({
    theme: {
      background: '#2c2c54',
      white: '#706fd3'
    },
    fontSize: 16,
    fontFamily: 'monospace',
    cursorBlink: true
  });
  const terminal = new LocalEchoController();
  const fitAddon = new FitAddon();
  __terminal.loadAddon(terminal);
  __terminal.loadAddon(fitAddon);

  useEffect(() => {
    // This runs twice. One for a rough fit, one for an exact fit
    setImmediate(() => fitAddon.fit());
    setTimeout(() => fitAddon.fit(), 1000);
    window.addEventListener('resize', () => fitAddon.fit());

    return () => window.removeEventListener('resize', fitAddon.fit);
  });

  terminal.addAutocompleteHandler(
    (index: number, tokens: string[], args: unknown[]): string[] => {
      if (index === 0) {
        return commandsList;
      }
      // const command = tokens[0];
      // if (command in commandsList) {
      //   return commands[command].params.map((p) => `--${p}`);
      // }
      return [];
    }
  );

  function stringToPrint(msg: string | object): string {
    if (typeof msg !== 'string') {
      const str = JSONbigNative.stringify(msg, null, 2);
      if (str !== '{}') {
        return str;
      }
    }
    return msg + ' ';
  }

  function write(msg: string | object): void {
    terminal.print(stringToPrint(msg));
    __terminal.scrollToBottom();
  }

  function writeln(msg: string | object): void {
    console.log(msg);
    terminal.println(stringToPrint(msg));
    __terminal.scrollToBottom();
  }

  async function handleCommand(line: string) {
    const params = yargsParser(line, { boolean: ['includeEvents'] });
    console.log(params);
    const cmd = params._[0];
    if (!cmd) {
      return;
    }
    if (!(commandsList.includes(cmd as CommandNames))) {
      writeln(`command ${cmd} not found!`);
      return;
    }
    await cli.execute(params as CommandsArgs, {
      info: (message?: any, ...optionalParams: any[]) => {
        console.log(message);
        writeln('');
        try {
          // 1421851
          // 95f6ce2c353b3fb1f6a7636f38883ddd
          write(message);
          optionalParams.forEach(value => writeln(value));
        } catch (e) {
          writeln(e);
        }
      },
      error: (message?: any, ...optionalParams: any[]) => {
        // console.error(message, optionalParams);
        writeln('');
        writeln('error:');
        writeln(message);
        optionalParams.forEach(value => writeln(value));
      },
      group: (...label: any[]) => {
        writeln('');
        writeln('--------------------------------------------------------');
      },
      groupEnd: () => {
        writeln('');
        writeln('--------------------------------------------------------');
      }
    });
  }

  async function prompt(): Promise<void> {
    while (isLive) {
      try {
        const line = await terminal.read(PROMPT);
        await handleCommand(line);
      } catch (e) {
        if (e === MANUAL_EXECUTION) {
          continue;
        }
        console.error(e);
      }
    }
  }

  function greeting(): void {
    GREETINGS.forEach((line) => terminal.println(line));
  }

  async function printAndExecuteCommand(command: string) {
    terminal.print(command);
    await handleCommand(command);
    terminal.abortRead(MANUAL_EXECUTION);
  }

  useEffect(() => {
    if (terminalRef.current) {
      // Creates the terminal within the container element.
      __terminal.open(terminalRef.current);
    }

    greeting();
    prompt();

    return () => {
      // When the component unmounts dispose of the terminal and all of its listeners.
      __terminal.dispose();
    };
  });

  return (
    <div className={className}>
      <div className="tutorial">
        <Transcript runCommand={printAndExecuteCommand}/>
      </div>
      <div
        className="terminal-container"
        style={{ padding: '1em', backgroundColor: '#2c2c54' }}
        ref={terminalRef}
      />
      <div className="commands-container">
        <CommandsPreset onCommandSelect={printAndExecuteCommand}/>
      </div>
    </div>
  );
}
