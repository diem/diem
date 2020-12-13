/// <reference types="react" />
interface CommandsListProps {
    runCommand: (command: string) => Promise<void>;
}
declare function Transcript({ runCommand }: CommandsListProps): JSX.Element;
export default Transcript;
