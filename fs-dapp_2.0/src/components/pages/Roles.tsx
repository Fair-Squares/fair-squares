import { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';
import { useAccountContext } from '../../contexts/Account_Context';
import React, { useEffect } from 'react';
import { useAppContext } from '../../contexts/AppContext';
import { useConcilSessionContext } from '../../contexts/CouncilSessionContext';
import BN from 'bn.js';
import { toUnit } from '../shared/utils';
import RolesApp from '../shared/modal';
import Referendum from '../shared/referendum';
import { Card, Col, Space } from 'antd';
import Identicon from '@polkadot/react-identicon';
//import { queryPublishedCredentials } from '../shared/Credentials';

export default function Roles() {
  const { api, blocks, selectedAccount, web3Name, credentials, dispatch } = useAppContext();
  const { role, balance, dispatch0 } = useAccountContext();
  const { role_in_session, dispatch1 } = useConcilSessionContext();

  useEffect(() => {
    if (!api || !selectedAccount) return;
    let address0 = selectedAccount.address;
    api.query.rolesModule.requestedRoles(address0, (data: any) => {
      let data0 = data.toHuman();
      if(!data0) return;
      console.log(`requested roles:${data}`);
      let r_session = data0.role.toString();

      dispatch1({ type: 'SET_ROLE_IN_SESSION', payload: r_session });
    });
  }, [blocks,selectedAccount,  dispatch1, api]);

  useEffect(() => {
    if (!api || !selectedAccount) return;
    let address0 = selectedAccount.address;

    dispatch0({ type: 'SET_ADDRESS', payload: address0 });
    api.query.rolesModule.accountsRolesLog(address0, (roles: string[]) => {
      let rl = roles;
      dispatch0({ type: 'SET_ROLES', payload: rl });
    });

    api.query.system.account(address0, ({ data: free }: { data: { free: BN } }) => {
      let { free: balance1 } = free;

      dispatch0({ type: 'SET_BALANCE', payload: balance1 });
    });

    api.query.council.members((who: any) => {
      dispatch1({ type: 'SET_COUNCIL_MEMBERS', payload: who as InjectedAccountWithMeta[] });
    });/*
    if (web3Name) {
      (async () => {
        let data_all = await queryPublishedCredentials(web3Name);
        if (data_all) {
          dispatch({ type: 'SET_ATTESTER', payload: data_all[0] });
          dispatch({ type: 'SET_CREDENTIALS', payload: data_all[1] });
        } else {
          dispatch({ type: 'SET_ATTESTER', payload: 'NONE' });
          dispatch({ type: 'SET_CREDENTIALS', payload: 'NONE' });
        }
      })();
    }*/
    api.query.backgroundCouncil.proposals((hash: string[]) => {
      if (hash.length > 0) {
        let hash0 = hash[0];

        api.query.backgroundCouncil.voting(hash0, (data: any) => {
          let data1 = data.toHuman();
          if (data1 !== null) {
            let yes = data1.ayes.length;
            let no = data1.nays.length;
            dispatch1({ type: 'SET_AYES', payload: yes });
            dispatch1({ type: 'SET_NAY', payload: no });
          }
        });
      }
    });
  }, [selectedAccount, api, dispatch0, dispatch1, dispatch, web3Name, blocks]);

  return (
    <div className="p-10">
      <Space direction="horizontal" size={100} style={{ display: 'flex' }} align="center">
        <Col span={70}>
          <Card title="Your wallet" size="default">
            {selectedAccount ? (
              <div>
                <p className="text-xl">
                  <Identicon
                    value={selectedAccount.address}
                    size={30}
                    theme={'polkadot'}
                    className="px-1 justify-center align-middle"
                  />
                  {' ' +
                    selectedAccount.meta.name +
                    ' | ' +
                    selectedAccount.address.slice(0, 6) +
                    '...' +
                    selectedAccount.address.slice(-6, -1)}
                </p>
                <p className="text-xl">
                  Balance: {!balance ? '0' : toUnit(balance, 3).toString()} FS
                </p>
                <p className="text-xl">
                  Credentials: {!credentials ? 'No Credentials' : credentials}
                </p>
              </div>
            ) : (
              <div>No Wallet Connected...</div>
            )}
          </Card>
        </Col>

        <Col span={50}>
          <Card title="Status" size="default">
            <h1>
              Your Roles:{' '}
              {!(role.length > 0)
                ? 'None'
                : role.map((value: string, index: number) => (
                    <p key={index} className="font-bold text-green-700 text-xl">
                      {value.toString()}
                    </p>
                  ))}
            </h1>
          </Card>
        </Col>

        <Col span={70}>
          <Card title="Requests status" size="default">
            <h1 className="flex flex-col px-4 ">
              Last Requested Role:{' '}
              <p className="font-bold text-red-800 text-xl">
                {!role_in_session || role.includes(role_in_session) ? 'None' : role_in_session}
              </p>
              <br />
              <div>
                <header>Referendum Status:</header>
                <p className="text-xl font-bold">
                  {!role_in_session || role.includes(role_in_session) ? (
                    'No active referendum'
                  ) : (
                    <Referendum />
                  )}
                </p>
              </div>
            </h1>
          </Card>
        </Col>

        <br />
        <br />

        <div className="flex flex-row items-start space-x-3 ">
          <RolesApp />
        </div>
      </Space>
    </div>
  );
}
