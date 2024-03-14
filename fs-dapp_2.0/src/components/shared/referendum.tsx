import React from 'react';
import { useConcilSessionContext } from '../../contexts/CouncilSessionContext';
import { useAccountContext } from '../../contexts/Account_Context';
import { Progress, Space } from 'antd';

function Referendum() {
  const { ayes, nay, role_in_session, council_members } = useConcilSessionContext();
  const { role } = useAccountContext();

  const yes = Number(((ayes / council_members.length) * 100).toFixed(1));

  return (
    <div>
      {
        <Space wrap>
          <Progress type="circle" percent={yes} size={80}></Progress>
          {ayes + nay > 0 ? (
            <Progress type="circle" percent={100 - yes} size={80} status="exception"></Progress>
          ) : (
            <p>No votes yet</p>
          )}
        </Space>
      }
    </div>
  );
}

export default Referendum;
