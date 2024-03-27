import { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';
import { useAccountContext } from '../../contexts/Account_Context';
import React, { useEffect,useState,useCallback } from 'react';
import { useAppContext } from '../../contexts/AppContext';
import { useConcilSessionContext } from '../../contexts/CouncilSessionContext';
import { DataType } from '@/src/contexts/types';
import BN from 'bn.js';
import { toUnit } from '../shared/utils';
import RolesApp from '../shared/modal';
import Referendum from '../shared/referendum';
import { Card, Col, Space } from 'antd';
import Identicon from '@polkadot/react-identicon';
import InfiniteScroll from "react-infinite-scroll-component";
import { Avatar,  Divider, List, Skeleton } from "antd";
import { Button } from 'flowbite-react';


export default function Council() {
  const { api, blocks, selectedAccount,accounts,  dispatch } = useAppContext();
  const { role, balance, dispatch0 } = useAccountContext();
  const { session_closed,approved,role_in_session,nay,ayes,council_members,selectedProposal,proposals, datas,dispatch1 } = useConcilSessionContext();
  
 
  const [hash0,setHash0] = useState<string[]>([]);

  function querryProposals(hash:string) {

    api?.query.backgroundCouncil.proposalOf(
      
      hash,(data_acc:any)=>{
        if(!data_acc) return;
        console.log(`datas:${data_acc}`);
        let acc = data_acc.toPrimitive().args.account as InjectedAccountWithMeta;
      let acc0 = data_acc.toPrimitive().args.account as string;
      const acc1 = accounts.find((account) => account.address === acc0);
      
        
      let acc_list = proposals;
      if (acc && !acc_list.includes(acc1 as InjectedAccountWithMeta) && acc1){
        acc_list.push(acc1);
        dispatch1({type:`SET_PROPOSALS`,payload:[]});                     
        dispatch1({type:`SET_PROPOSALS`,payload:acc_list}); 
        
        
        
        api.query.rolesModule.requestedRoles(acc1.address,(Prop_raw:any)=>{
          let Prop = Prop_raw.toHuman();
      if(!Prop) return;
      let r_session = Prop.role.toString();
      let status = Prop.approved.toString();
      let referendum = Prop.sessionClosed.toString();
      let hash = Prop.proposalHash.toString();
      let dtype:DataType={name:acc1.meta.name,role:r_session,address:acc.address,status,referendum,hash};
      let tdata=datas
      
      tdata.push(dtype);
      dispatch1({type:`SET_DATAS`,payload:tdata}); 
        })       
       
      }        
    }
    );
  }


  const update = useCallback(() =>{    
    let bb =hash0.map((x)=>querryProposals(x));
    return bb},[hash0,blocks]);




   useEffect(() => {
    if (!api || !selectedAccount) return;

    api.query.backgroundCouncil.proposals((hash: string[]) => {
      if (hash.length > 0) {
        
        setHash0(hash);
        update() 
        let tdata0:DataType[]=[];
        datas.forEach(elemnt=>{
          if (hash0.includes(elemnt.hash) && !tdata0.includes(elemnt) && elemnt.status!==`AWAITING`){
            tdata0.push(elemnt);
            console.log(`Number of active proposals: ${tdata0.length}`)
          }
          if (hash0.includes(elemnt.hash) && !tdata0.includes(elemnt)){
            tdata0.push(elemnt);
            console.log(`Number of active proposals: ${tdata0.length}`)
          }else{tdata0=datas}
        })
        dispatch1({type:`SET_DATAS`,payload:tdata0});      
        
        
      }    
    });   
   

  }, [hash0,selectedAccount,api]);

 

  return(<div id="scrollableDiv"
  style={{
    height: 400,
    overflow: 'auto',
    padding: '0 16px',
    border: '1px solid rgba(140, 140, 140, 0.35)',
  }}>
    
       <List
          dataSource={datas}
          
          renderItem={item => (
            <Card 
            hoverable
      style={{ width: 300, height:120}}>
            <List.Item key={item.address}>
              <List.Item.Meta
                title={<p>{item.name}</p>}
                description={<div><p>Requested Role: {item.role}</p><p>Request Status: {item.status}</p></div>}
              />
              <div>Content</div>
            </List.Item>
            </Card>
          )}

        />

   
    
  </div>);

  }