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
import InfiniteScroll from "react-infinite-scroll-component";
import { Avatar,  Divider, List, Skeleton } from "antd";

export default function Council() {
  const { api, blocks, selectedAccount,  dispatch } = useAppContext();
  const { role, balance, dispatch0 } = useAccountContext();
  const { session_closed,approved,role_in_session,nay,ayes,council_members,selectedProposal,proposals, dispatch1 } = useConcilSessionContext();
 
   useEffect(() => {
    if (!api || !selectedAccount) return;

    api.query.backgroundCouncil.proposals((hash: string[]) => {
      if (hash.length > 0) {
        let hash0 = hash[0];

        api.query.backgroundCouncil.proposalOf(hash0, (data: any) => {
          let data1 = data;
          if (data1 !== null) {
            console.log(`Proposal Data: ${data1}`);
          }
        });
      }
    });

  }, [selectedAccount, blocks, dispatch1, api]);
  return(<div>
    Council Page
  </div>);

  }