import axios from 'axios';
import type { KeyringPair } from '@polkadot/keyring/types';
import * as Kilt from '@kiltprotocol/sdk-js';

export async function queryAccountWeb3Name(
  lookupAccountAddress: KeyringPair['address'],
): Promise<Kilt.Did.Web3Name | undefined> {
  const api = Kilt.ConfigService.get('api');

  const encodedLinkedDetails = await api.call.did.queryByAccount(
    Kilt.Did.accountToChain(lookupAccountAddress),
  );
  const { web3Name } = Kilt.Did.linkedInfoFromChain(encodedLinkedDetails);
  if (web3Name) {
    console.log(`web3name for account "${lookupAccountAddress}" -> "${web3Name}"`);
  } else {
    console.log(`Account "${lookupAccountAddress}" does not have a linked web3name.`);
  }

  return web3Name;
}

export async function queryPublishedCredentials(web3Name: string | undefined) {
  if (!web3Name) return;

  const api = await Kilt.connect('wss://spiritnet.kilt.io/');
  const encodedDetails = await api.call.did.queryByWeb3Name(web3Name);

  // This function will throw if web3Name does not exist
  const {
    document: { uri },
  } = Kilt.Did.linkedInfoFromChain(encodedDetails);
  console.log(`My name is ${web3Name} and this is my DID: "${uri}"`);

  const DidDocument = await Kilt.Did.resolve(uri);
  console.log(`John Doe's DID Document:`);
  console.log(JSON.stringify(DidDocument, null, 2));

  const endpoints = DidDocument?.document?.service;
  if (!endpoints) {
    console.log('No endpoints for the DID.');
    return [];
  }

  console.log('Endpoints:');
  console.log(JSON.stringify(endpoints, null, 2));

  const {
    data: [{ credential }],
  } = await axios.get<Kilt.KiltPublishedCredentialCollectionV1>(endpoints[0].serviceEndpoint[0]);
  try {
    const { attester, revoked } = await Kilt.Credential.verifyCredential(credential);

    // Verify that the credential is not revoked. Exception caught by the catch {} block below.
    if (revoked) {
      throw new Error('The credential has been revoked, hence it is not valid.');
    }

    let res = [attester, JSON.stringify(credential, null, 2)];
    return res;
  } catch {
    console.log(`${web3Name}'s credential is not valid.`);
  }

  await Kilt.disconnect();
}
export default queryPublishedCredentials;
