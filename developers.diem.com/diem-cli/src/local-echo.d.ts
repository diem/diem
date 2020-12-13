declare module 'local-echo' {
  import { ITerminalAddon, Terminal } from 'xterm';

  declare type AutoCompleteFunc = (
    index: number,
    tokens: string[],
    args: unknown[]
  ) => string[];

  export default class LocalEchoController extends ITerminalAddon {
    read(prompt: string): Promise<string>;
    addAutocompleteHandler(callback: AutoCompleteFunc, args?: unknown[]);
    print(msg: string);
    println(line: string): void;
    activate(terminal: Terminal);
    dispose();
    abortRead(reason: string);
  }
}
