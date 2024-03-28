import React, { useEffect } from 'react';
import { useAppContext } from '../../contexts/AppContext';
import { useAccountContext } from '../../contexts/Account_Context';
import { Chart as ChartJS, ArcElement, Tooltip, Legend } from 'chart.js';
import { Pie } from 'react-chartjs-2';
import BN from 'bn.js';
import { toUnit } from '../shared/utils';
import { NavLink } from 'react-router-dom';
ChartJS.register(ArcElement, Tooltip, Legend);
const treasury_address = '5EYCAe5h8JVpdkpBnytXdq1R8u69C3a7zi7iuipxUc8NVHqh';

export default function Dashboard() {
  const {
    api,
    blocks,
    total_users_nbr,
    inv_nbr,
    seller_nbr,
    awaiting_seller_nbr,
    servicer_nbr,
    awaiting_servicer_nbr,
    tenant_nbr,
    treasury_balance,
    selectedAccount,
    dispatch,
  } = useAppContext();
  const { role } = useAccountContext();

  useEffect(() => {
    if (!api) return;

    api.query.system.account(treasury_address, ({ data: free }: { data: { free: BN } }) => {
      let { free: balance1 } = free;
      dispatch({ type: 'SET_TREASURY_BALANCE', payload: balance1 });
    });

    api.query.rolesModule.investorLog.entries((data: any) => {
      dispatch({ type: 'SET_INVESTORS_NBR', payload: data.length });
    });
    api.query.rolesModule.tenantLog.entries((data: any) => {
      dispatch({ type: 'SET_TENANTS_NBR', payload: data.length });
    });

    api.query.rolesModule.sellerApprovalList((data: any) => {
      dispatch({ type: 'SET_A_SELLERS_NBR', payload: data.length });
    });

    api.query.rolesModule.houseSellerLog.entries((data: []) => {
      dispatch({ type: 'SET_SELLERS_NBR', payload: data.length });
    });

    api.query.rolesModule.servicerApprovalList((data: any) => {
      dispatch({ type: 'SET_A_SERVICER_NBR', payload: data.length });
    });

    api.query.rolesModule.servicerLog.entries((data: any) => {
      dispatch({ type: 'SET_SERVICER_NBR', payload: data.length });
    });

    api.query.rolesModule.totalMembers((data: number) => {
      let data1 = Number(data.toString());
      dispatch({ type: 'SET_TOTAL', payload: data1 });
    });
  }, [blocks, api, dispatch]);

  const maxRoles = Number(api?.consts.rolesModule.maxRoles);

  const data = {
    labels: [
      'Investors',
      'Servicers',
      'Tenants',
      'Sellers',
      'Awaiting_Sellers',
      'Awaiting_Servicers ',
    ],
    datasets: [
      {
        label: '# of roles',
        data: [
          inv_nbr,
          servicer_nbr,
          tenant_nbr,
          seller_nbr,
          awaiting_seller_nbr,
          awaiting_servicer_nbr,
        ],
        backgroundColor: [
          'rgba(255, 99, 132, 1)',
          'rgba(54, 162, 235, 1)',
          'rgba(255, 206, 86, 1)',
          'rgba(75, 192, 192, 1)',
          'rgba(255, 0, 255, 1)',
          'rgba(0, 0, 151, 1)',
        ],
        borderColor: [
          'rgba(255, 99, 132, 1)',
          'rgba(54, 162, 235, 1)',
          'rgba(255, 206, 86, 1)',
          'rgba(75, 192, 192, 1)',
          'rgba(255, 0, 255, 1)',
          'rgba(0, 0, 151, 1)',
        ],
        borderWidth: 1,
      },
    ],
  };

  return (
    <div className="flex flex-row justify-between p-6">
      <div>
        <h1 className="text-3xl text-slate-700 font-bold">DASHBOARD</h1>
        <p className="text-xl font-bold">
          House Fund: {!treasury_balance ? '0' : toUnit(treasury_balance, 3).toString()} FS
        </p>
        <p className="text-xl font-bold">Total Number of Users: {total_users_nbr}</p>
      </div>

      <div style={{ width: 500 }}>
        <Pie data={data} />
      </div>
      <div className="flex flex-col">
        {!selectedAccount ? (
          <p className="font-bold text-slate-900 text-3xl">Wallet not connected</p>
        ) : (
          <NavLink
            to="/roles"
            className={
              'rounded-md bg-neutral-900  px-4 py-1 ' +
              (role.length < maxRoles ? 'animate-pulse text-yellow-300' : 'text-white')
            }
          >
            Role Status
          </NavLink>
        )}
      </div>
    </div>
  );
}
