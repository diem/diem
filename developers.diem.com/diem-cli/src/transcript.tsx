import React, { useState } from 'react';
import LibraUtils from '@libra-opensource/libra-client-sdk-typescript/dist/libraUtils';
import { bytesToHexString } from '@libra-opensource/libra-client-sdk-typescript/dist/utils/bytes';

interface CommandsListProps {
  runCommand: (command: string) => Promise<void>;
}

function getRandomBytes(len: number) {
  return typeof window !== 'undefined'
    ? Array.from(crypto.getRandomValues(new Uint8Array(len))).map(m=>('0'+m.toString(16)).slice(-2)).join('')
    : [];
}

function Transcript({ runCommand }: CommandsListProps) {
  const [address, setAddress] = useState('');

  function accountAddressFromSeed(seed: string) {
    const accountKeys = LibraUtils.generateAccountKeys(seed);
    return accountKeys.accountAddress;
  }

  return (
    <>
      <div>
        <ul>
          <li key="step1" className="list-group-item">
            <div className="step-headline">Step #1:</div>
            <div className="step-description">Let's start with generating a set of private and public keys.</div>
            <div className="step-description"><button onClick={async () => {await runCommand('generate_keys')}}>Generate keys</button></div>
          </li>
          <li key="step2" className="list-group-item">
            <div className="step-headline">Step #2:</div>
            <div className="step-description">Now, take the private key and use it to seed a new account.</div>
            <form
              onSubmit={async (e) => {
                e.preventDefault();
                const formData = new FormData(e.currentTarget);
                const seed = formData.get('seed');
                setAddress(bytesToHexString(accountAddressFromSeed(seed as string)));
                await runCommand(`create_account --seed ${seed}`);
              }}
            >
              <div className="form-group">
                <label>Seed</label>
                <input type="text" name="seed" className="form-typed-input" />
              </div>
              <div className="step-description"><button type="submit">Create account</button></div>
            </form>
          </li>
          <li key="step3" className="list-group-item">
            <div className="step-headline">Step #3:</div>
            <div className="step-description">Your Testnet account starts with a 1 Coin1 balance, let's see that.</div>
            <form
              onSubmit={async (e) => {
                e.preventDefault();
                const formData = new FormData(e.currentTarget);
                await runCommand(`get_account --address ${formData.get('accountAddress')}`);
              }}
            >
              <div className="form-group">
                <label>Account address</label>
                <input type="text" value={address} name="accountAddress" className="form-typed-input" />
              </div>
              <div className="step-description"><button type="submit">Check</button></div>
            </form>
          </li>
          <li key="step4" className="list-group-item">
            <div className="step-headline">Step #4:</div>
            <div className="step-description">Let's craft a receiving address using LIP-5 format (bech32). For that, we will use the account address and a random generated subaddress.</div>
            <form
              onSubmit={async (e) => {
                e.preventDefault();
                const formData = new FormData(e.currentTarget);
                await runCommand(`address_to_bech32 --address ${formData.get('accountAddress')}  --subAddress ${formData.get('subaddress')} --hrp tlb`);
              }}
            >
              <div className="form-group">
                <label>Account address</label>
                <input type="text" value={address} name="accountAddress" className="form-typed-input" />
              </div>
              <div className="form-group">
                <label>Subaddress</label>
                <input value={getRandomBytes(8)} type="text" name="subaddress" className="form-typed-input" />
              </div>
              <div className="step-description"><button type="submit">Format</button></div>
            </form>
          </li>
          <li key="step5" className="list-group-item">
            <div className="step-headline">Step #5:</div>
            <div className="step-description">Now go to <a href="https://developers.diem.com/docs/wallet-app/public-demo-wallet#use-the-demo-wallet" target="_blank">Diem Reference Wallet</a> (Demo wallet) and send yourself some coins...</div>
          </li>
          <li key="step6" className="list-group-item">
            <div className="step-headline">Step #6:</div>
            <div className="step-description">We can now validate the account balance using the get_account command.</div>
            <form
              onSubmit={async (e) => {
                e.preventDefault();
                const formData = new FormData(e.currentTarget);
                await runCommand(`get_account --address ${formData.get('accountAddress')}`);
              }}
            >
              <div className="form-group">
                <label>Account address</label>
                <input type="text" value={address} name="accountAddress" className="form-typed-input" />
              </div>
              <div className="step-description"><button type="submit">Check</button></div>
            </form>
          </li>
        </ul>
      </div>
    </>
  );
}

export default Transcript;
