var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
import React, { useState } from 'react';
import LibraUtils from '@libra-opensource/libra-client-sdk-typescript/dist/libraUtils';
import { bytesToHexString } from '@libra-opensource/libra-client-sdk-typescript/dist/utils/bytes';
function getRandomBytes(len) {
    return typeof window !== 'undefined'
        ? Array.from(crypto.getRandomValues(new Uint8Array(len))).map(m => ('0' + m.toString(16)).slice(-2)).join('')
        : [];
}
function Transcript({ runCommand }) {
    const [address, setAddress] = useState('');
    function accountAddressFromSeed(seed) {
        const accountKeys = LibraUtils.generateAccountKeys(seed);
        return accountKeys.accountAddress;
    }
    return (React.createElement(React.Fragment, null,
        React.createElement("div", null,
            React.createElement("ul", null,
                React.createElement("li", { key: "step1", className: "list-group-item" },
                    React.createElement("div", { className: "step-headline" }, "Step #1:"),
                    React.createElement("div", { className: "step-description" }, "Let's start with generating a set of private and public keys."),
                    React.createElement("div", { className: "step-description" },
                        React.createElement("button", { onClick: () => __awaiter(this, void 0, void 0, function* () { yield runCommand('generate_keys'); }) }, "Generate keys"))),
                React.createElement("li", { key: "step2", className: "list-group-item" },
                    React.createElement("div", { className: "step-headline" }, "Step #2:"),
                    React.createElement("div", { className: "step-description" }, "Now, take the private key and use it to seed a new account."),
                    React.createElement("form", { onSubmit: (e) => __awaiter(this, void 0, void 0, function* () {
                            e.preventDefault();
                            const formData = new FormData(e.currentTarget);
                            const seed = formData.get('seed');
                            setAddress(bytesToHexString(accountAddressFromSeed(seed)));
                            yield runCommand(`create_account --seed ${seed}`);
                        }) },
                        React.createElement("div", { className: "form-group" },
                            React.createElement("label", null, "Seed"),
                            React.createElement("input", { type: "text", name: "seed", className: "form-typed-input" })),
                        React.createElement("div", { className: "step-description" },
                            React.createElement("button", { type: "submit" }, "Create account")))),
                React.createElement("li", { key: "step3", className: "list-group-item" },
                    React.createElement("div", { className: "step-headline" }, "Step #3:"),
                    React.createElement("div", { className: "step-description" }, "Your Testnet account starts with a 1 Coin1 balance, let's see that."),
                    React.createElement("form", { onSubmit: (e) => __awaiter(this, void 0, void 0, function* () {
                            e.preventDefault();
                            const formData = new FormData(e.currentTarget);
                            yield runCommand(`get_account --address ${formData.get('accountAddress')}`);
                        }) },
                        React.createElement("div", { className: "form-group" },
                            React.createElement("label", null, "Account address"),
                            React.createElement("input", { type: "text", value: address, name: "accountAddress", className: "form-typed-input" })),
                        React.createElement("div", { className: "step-description" },
                            React.createElement("button", { type: "submit" }, "Check")))),
                React.createElement("li", { key: "step4", className: "list-group-item" },
                    React.createElement("div", { className: "step-headline" }, "Step #4:"),
                    React.createElement("div", { className: "step-description" }, "Let's craft a receiving address using LIP-5 format (bech32). For that, we will use the account address and a random generated subaddress."),
                    React.createElement("form", { onSubmit: (e) => __awaiter(this, void 0, void 0, function* () {
                            e.preventDefault();
                            const formData = new FormData(e.currentTarget);
                            yield runCommand(`address_to_bech32 --address ${formData.get('accountAddress')}  --subAddress ${formData.get('subaddress')} --hrp tlb`);
                        }) },
                        React.createElement("div", { className: "form-group" },
                            React.createElement("label", null, "Account address"),
                            React.createElement("input", { type: "text", value: address, name: "accountAddress", className: "form-typed-input" })),
                        React.createElement("div", { className: "form-group" },
                            React.createElement("label", null, "Subaddress"),
                            React.createElement("input", { value: getRandomBytes(8), type: "text", name: "subaddress", className: "form-typed-input" })),
                        React.createElement("div", { className: "step-description" },
                            React.createElement("button", { type: "submit" }, "Format")))),
                React.createElement("li", { key: "step5", className: "list-group-item" },
                    React.createElement("div", { className: "step-headline" }, "Step #5:"),
                    React.createElement("div", { className: "step-description" },
                        "Now go to ",
                        React.createElement("a", { href: "https://developers.diem.com/docs/wallet-app/public-demo-wallet#use-the-demo-wallet", target: "_blank" }, "Diem Reference Wallet"),
                        " (Demo wallet) and send yourself some coins...")),
                React.createElement("li", { key: "step6", className: "list-group-item" },
                    React.createElement("div", { className: "step-headline" }, "Step #6:"),
                    React.createElement("div", { className: "step-description" }, "We can now validate the account balance using the get_account command."),
                    React.createElement("form", { onSubmit: (e) => __awaiter(this, void 0, void 0, function* () {
                            e.preventDefault();
                            const formData = new FormData(e.currentTarget);
                            yield runCommand(`get_account --address ${formData.get('accountAddress')}`);
                        }) },
                        React.createElement("div", { className: "form-group" },
                            React.createElement("label", null, "Account address"),
                            React.createElement("input", { type: "text", value: address, name: "accountAddress", className: "form-typed-input" })),
                        React.createElement("div", { className: "step-description" },
                            React.createElement("button", { type: "submit" }, "Check"))))))));
}
export default Transcript;
