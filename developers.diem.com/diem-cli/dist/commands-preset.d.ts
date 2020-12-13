/// <reference types="react" />
interface CommandsListProps {
    onCommandSelect: (command: string) => void;
}
declare function CommandsPreset({ onCommandSelect }: CommandsListProps): JSX.Element;
export default CommandsPreset;
