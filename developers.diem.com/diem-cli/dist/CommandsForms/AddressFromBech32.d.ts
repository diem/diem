/// <reference types="react" />
declare function AddressFromBech32({ onSubmit, }: {
    onSubmit: (command: string) => void;
}): JSX.Element;
export default AddressFromBech32;
