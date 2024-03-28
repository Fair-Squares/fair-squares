import React, { useEffect, useState } from 'react';
import { Button, Drawer } from 'antd';
import { useAccountContext } from '../../contexts/Account_Context';
import { useAppContext } from '../../contexts/AppContext';
import { ROLES } from '../../contexts/types';
import { web3FromAddress } from '@polkadot/extension-dapp';
import { Toast } from 'flowbite-react';
import { NotificationTwoTone, WarningTwoTone } from '@ant-design/icons';

const RolesApp: React.FC = () => {
  const [open, setOpen] = useState(false);
  const [event, setEvents] = useState('No Roles');
  const [showToast, setShowToast] = useState(false);
  const [warning, setWarning] = useState(false);
  const { api, selectedAccount } = useAppContext();
  const { role } = useAccountContext();

  const showDrawer = () => {
    setOpen(true);
  };

  const onClose = () => {
    setOpen(false);
  };

  const getRole = async (num: number) => {
    if (!api || !selectedAccount || ROLES.includes(role.toString())) {
      console.log('No Roles possible!');
    } else {
      let who = selectedAccount.address;
      const tx = await api.tx.rolesModule.setRole(who, ROLES[num].toString());
      const fees = await tx.paymentInfo(who);
      const injector = await web3FromAddress(who);
      tx.signAndSend(who, { signer: injector.signer }, ({ status, events, dispatchError }) => {
        if (dispatchError && status.isInBlock) {
          if (dispatchError.isModule) {
            console.log(`Current status: ${status.type}`);
            // for module errors, we have the section indexed, lookup
            const decoded = api.registry.findMetaError(dispatchError.asModule);
            const { docs, name, section } = decoded;
            setEvents(name.toString());
            setShowToast(true);
            setWarning(true);

            console.log(`${section}.${name}: ${docs.join(' ')}`);
          }
        } else if (status.isInBlock) {
          console.log(`Fees: ${fees.partialFee}`);
          console.log(`Current status: ${status.type}`);
          events.forEach(({ event: { method, section, data } }) => {
            if (section.toString().includes('rolesModule')) {
              let meth = method.toString() + '\n';
              let payed = '\n' + fees.partialFee.toString() + ' FS';
              setEvents(`${meth} =>Paid fees: ${payed} `);
              setShowToast(true);
              setWarning(false);
            }
          });
        } else {
          console.log(`Current status: ${status.type}`);
        }
      });
    }
  };

  useEffect(() => {
    if (event !== 'No Roles') console.log(event);
  }, [event]);

  return (
    <p className="flex-col space-y-2">
      <Button
        type="primary"
        className="bg-blue-600 text-white font-bold py-2 pb-10 text-xl"
        onClick={showDrawer}
      >
        Select a Role
      </Button>
      {!(showToast === false) ? (
        <Toast>
          <div
            className={
              'shadow-md rounded-md flex  text-white text-base items-center justify-normal ' +
              (warning === true ? ' bg-red-500 animate-bounce ' : ' bg-green-600  animate-pulse')
            }
          >
            <div>
              {!(warning === true) ? (
                <NotificationTwoTone twoToneColor="#52c41a" className="h-8 w-8" />
              ) : (
                <WarningTwoTone twoToneColor="#eb2f96" className="h-8 w-8" />
              )}
            </div>
            <div className="p-2">{event}</div>
            <Toast.Toggle
              onClick={() => {
                setShowToast(false);
              }}
            />
          </div>
        </Toast>
      ) : (
        <div className=" p-2"> </div>
      )}

      <Drawer title="Basic Drawer" placement="right" onClose={onClose} open={open}>
        <p>
          <Button
            type="primary"
            className="bg-pink-600 rounded"
            onClick={() => {
              getRole(0);
            }}
          >
            {ROLES[0].toString()}
          </Button>
        </p>
        <br />
        <p>
          <Button type="primary" className="bg-pink-600 rounded" onClick={() => getRole(1)}>
            {ROLES[1].toString()}
          </Button>
        </p>
        <br />
        <p>
          <Button type="primary" className="bg-pink-600 rounded" onClick={() => getRole(2)}>
            {ROLES[2].toString()}
          </Button>
        </p>
        <br />
        <p>
          <Button type="primary" className="bg-pink-600 rounded" onClick={() => getRole(3)}>
            {ROLES[3].toString()}
          </Button>
        </p>
      </Drawer>
    </p>
  );
};

export default RolesApp;
