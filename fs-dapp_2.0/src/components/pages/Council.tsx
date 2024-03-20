import { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';
import { useAccountContext } from '../../contexts/Account_Context';
import React, { useEffect,useState } from 'react';
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

interface DataType {
  name: string;
  role: string;
  address:string;  
}
export default function Council() {
  const { api, blocks, selectedAccount,  dispatch } = useAppContext();
  const { role, balance, dispatch0 } = useAccountContext();
  const { session_closed,approved,role_in_session,nay,ayes,council_members,selectedProposal,proposals, dispatch1 } = useConcilSessionContext();
  
  const [loading, setLoading] = useState(false);
  const [data, setData] = useState<DataType[]>([]);

  function querryProposals(hash:string) {

    api?.query.backgroundCouncil.proposalOf(
      
      hash,(data1:any)=>{
      let data0 = data1.toPrimitive().args.account as InjectedAccountWithMeta;    
      let data = proposals;
      if (data0 && !data.includes(data0)){
        data.push(data0);                    
        dispatch1({type:`SET_PROPOSALS`,payload:data}); 
       
      }        
    }
    );
  }
  function updateProposals(){
    api?.query.backgroundCouncil.proposals((hash: string[]) => {
      if (hash.length > 0) {
        hash.map((x)=>querryProposals(x));
      }
      
    
    });
  }

  

   useEffect(() => {
    if (!api || !selectedAccount) return;

    updateProposals()    

  }, [selectedAccount, blocks, dispatch1, api]);
  return(<div id="scrollableDiv"
  style={{
    height: 400,
    overflow: 'auto',
    padding: '0 16px',
    border: '1px solid rgba(140, 140, 140, 0.35)',
  }}>
    <InfiniteScroll
     dataLength={proposals.length}
     next={updateProposals}
     hasMore={proposals.length < 50}
     loader={<Skeleton avatar paragraph={{ rows: 1 }} active />}
     endMessage={<Divider plain>It is all, nothing more 🤐</Divider>}
     scrollableTarget="scrollableDiv"
    >
       <List
          dataSource={data}
          renderItem={item => (
            <List.Item key={item.address}>
              <List.Item.Meta
                title={<a href="https://ant.design">{item.name}</a>}
                description={item.role}
              />
              <div>Content</div>
            </List.Item>
          )}
        />

    </InfiniteScroll>
    Council Page
  </div>);

  }