var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
import * as React from 'react';
import { useEffect } from 'react';
import { Terminal } from 'xterm-dfuzr';
import LocalEchoController from 'local-echo';
import { FitAddon } from 'xterm-addon-fit';
import yargsParser from 'yargs-parser';
import 'xterm/css/xterm.css';
import CommandsPreset from './commands-preset';
import * as cli from '@libra-opensource/libra-client-sdk-typescript/dist/cli';
import { commandsList } from '@libra-opensource/libra-client-sdk-typescript/dist/cli/commands/command';
import JSONBigint from 'json-bigint';
import Transcript from './transcript';
const JSONbigNative = JSONBigint({
    useNativeBigInt: true
});
const MANUAL_EXECUTION = 'MANUAL_EXECUTION';
export default function DiemTerminal({ className }) {
    const PROMPT = '≋Diem> ';
    const GREETINGS = [
        'Welcome to Diem Shell 💁‍♀️',
        'This is a fully local Diem testnet client, running entirely on your browser!',
        ''
    ];
    const isLive = true;
    const terminalRef = React.createRef();
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
    terminal.addAutocompleteHandler((index, tokens, args) => {
        if (index === 0) {
            return commandsList;
        }
        // const command = tokens[0];
        // if (command in commandsList) {
        //   return commands[command].params.map((p) => `--${p}`);
        // }
        return [];
    });
    function stringToPrint(msg) {
        if (typeof msg !== 'string') {
            const str = JSONbigNative.stringify(msg, null, 2);
            if (str !== '{}') {
                return str;
            }
        }
        return msg + ' ';
    }
    function write(msg) {
        terminal.print(stringToPrint(msg));
        __terminal.scrollToBottom();
    }
    function writeln(msg) {
        console.log(msg);
        terminal.println(stringToPrint(msg));
        __terminal.scrollToBottom();
    }
    function handleCommand(line) {
        return __awaiter(this, void 0, void 0, function* () {
            const params = yargsParser(line, { boolean: ['includeEvents'] });
            console.log(params);
            const cmd = params._[0];
            if (!cmd) {
                return;
            }
            if (!(commandsList.includes(cmd))) {
                writeln(`command ${cmd} not found!`);
                return;
            }
            yield cli.execute(params, {
                info: (message, ...optionalParams) => {
                    console.log(message);
                    writeln('');
                    try {
                        // 1421851
                        // 95f6ce2c353b3fb1f6a7636f38883ddd
                        write(message);
                        optionalParams.forEach(value => writeln(value));
                    }
                    catch (e) {
                        writeln(e);
                    }
                },
                error: (message, ...optionalParams) => {
                    // console.error(message, optionalParams);
                    writeln('');
                    writeln('error:');
                    writeln(message);
                    optionalParams.forEach(value => writeln(value));
                },
                group: (...label) => {
                    writeln('');
                    writeln('--------------------------------------------------------');
                },
                groupEnd: () => {
                    writeln('');
                    writeln('--------------------------------------------------------');
                }
            });
        });
    }
    function prompt() {
        return __awaiter(this, void 0, void 0, function* () {
            while (isLive) {
                try {
                    const line = yield terminal.read(PROMPT);
                    yield handleCommand(line);
                }
                catch (e) {
                    if (e === MANUAL_EXECUTION) {
                        continue;
                    }
                    console.error(e);
                }
            }
        });
    }
    function greeting() {
        GREETINGS.forEach((line) => terminal.println(line));
    }
    function printAndExecuteCommand(command) {
        return __awaiter(this, void 0, void 0, function* () {
            terminal.print(command);
            yield handleCommand(command);
            terminal.abortRead(MANUAL_EXECUTION);
        });
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
    return (React.createElement("div", { className: className },
        React.createElement("div", { className: "tutorial" },
            React.createElement(Transcript, { runCommand: printAndExecuteCommand })),
        React.createElement("div", { className: "terminal-container", style: { padding: '1em', backgroundColor: '#2c2c54' }, ref: terminalRef }),
        React.createElement("div", { className: "commands-container" },
            React.createElement(CommandsPreset, { onCommandSelect: printAndExecuteCommand }))));
}
